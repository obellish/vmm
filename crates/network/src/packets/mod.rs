#[path = "client.rs"]
pub mod clientbound;
#[path = "server.rs"]
pub mod serverbound;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{self, Cursor, Read, Write},
	net::TcpStream,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
};

use byteorder::{BigEndian, ReadBytesExt as _, WriteBytesExt as _};
use flate2::{Compression, bufread::ZlibDecoder, read, write::ZlibEncoder};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};
use vmm_text::TextComponent;

use self::serverbound::{
	SAcknowledgeFinishConfiguration, SChatCommand, SChatMessage, SClientInformation,
	SCommandSuggestionsRequest, SHandshake, SKeepAlive, SLoginAcknowledged, SLoginPluginResponse,
	SLoginStart, SPing, SPlayerAbilities, SPlayerAction, SPlayerCommand, SPlayerRotation,
	SPluginMessage, SRequest, SSetCreativeModeSlot, SSetHeldItem, SSetPlayerOnGround,
	SSetPlayerPosition, SSetPlayerPositionAndRotation, SSwingArm, SUnknown, SUpdateSign,
	SUseItemOn, ServerBoundPacket,
};
use super::{NbtCompound, NetworkState};

pub const COMPRESSION_THRESHOLD: usize = 256;

#[derive(Debug)]
pub struct SlotData {
	pub item_id: i32,
	pub item_count: i8,
	pub nbt: Option<NbtCompound>,
}

#[derive(Debug, Clone)]
pub struct PlayerProperty {
	pub name: String,
	pub value: String,
	pub signature: Option<String>,
}

#[derive(Debug)]
pub struct PalettedContainer {
	pub bits_per_entry: u8,
	pub palette: Option<Vec<i32>>,
	pub data_array: Vec<u64>,
}

pub struct PacketEncoder {
	buffer: Vec<u8>,
	packet_id: u32,
}

impl PacketEncoder {
	fn new(buffer: Vec<u8>, packet_id: u32) -> Self {
		trace!("encoding packet with id {packet_id:#02x}");
		Self { buffer, packet_id }
	}

	fn varint(value: i32) -> Vec<u8> {
		let mut value = value as u32;
		let mut buf = Vec::new();
		loop {
			let mut temp = (value & 0b1111_1111) as u8;
			value >>= 7;
			if !matches!(value, 0) {
				temp |= 0b1000_0000;
			}

			buf.push(temp);
			if matches!(value, 0) {
				break buf;
			}
		}
	}

	pub fn write_compressed(&self, mut w: impl Write) -> io::Result<()> {
		let packet_id = Self::varint(self.packet_id as i32);
		let data = [packet_id.as_slice(), self.buffer.as_slice()].concat();
		if self.buffer.len() < COMPRESSION_THRESHOLD {
			let packet_length = Self::varint((1 + data.len()) as i32);

			w.write_all(&packet_length)?;

			w.write_all(&[0])?;
			w.write_all(&data)?;
		} else {
			let data_length = Self::varint(data.len() as i32);
			let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
			encoder.write_all(&data)?;
			let compressed = encoder.finish()?;
			let packet_length = Self::varint((data_length.len() + compressed.len()) as i32);

			w.write_all(&packet_length)?;
			w.write_all(&data_length)?;
			w.write_all(&compressed)?;
		}

		Ok(())
	}

	pub fn write_uncompressed(&self, mut w: impl Write) -> io::Result<()> {
		let packet_id = Self::varint(self.packet_id as i32);
		let length = Self::varint((self.buffer.len() + packet_id.len()) as i32);

		w.write_all(&length)?;
		w.write_all(&packet_id)?;
		w.write_all(&self.buffer)?;

		Ok(())
	}
}

#[derive(Debug)]
pub enum PacketDecodeError {
	Io(io::Error),
	FromUtf8(std::string::FromUtf8Error),
	Nbt(nbt::Error),
}

impl Display for PacketDecodeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
			Self::FromUtf8(e) => Display::fmt(&e, f),
			Self::Nbt(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for PacketDecodeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::FromUtf8(e) => Some(e),
			Self::Nbt(e) => Some(e),
		}
	}
}

impl From<io::Error> for PacketDecodeError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<std::string::FromUtf8Error> for PacketDecodeError {
	fn from(value: std::string::FromUtf8Error) -> Self {
		Self::FromUtf8(value)
	}
}

impl From<nbt::Error> for PacketDecodeError {
	fn from(value: nbt::Error) -> Self {
		Self::Nbt(value)
	}
}

#[derive(Debug)]
pub enum PacketEncodeError {}

impl Display for PacketEncodeError {
	#[allow(clippy::uninhabited_references)]
	fn fmt(&self, _: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for PacketEncodeError {}

pub trait PacketDecoderExt: Read + Sized {
	fn read_unsigned_byte(&mut self) -> DecodeResult<u8> {
		Ok(self.read_u8()?)
	}

	fn read_byte(&mut self) -> DecodeResult<i8> {
		Ok(self.read_i8()?)
	}

	fn read_bytes(&mut self, bytes: usize) -> DecodeResult<Vec<u8>> {
		let mut read = vec![0; bytes];
		self.read_exact(&mut read)?;
		Ok(read)
	}

	fn read_long(&mut self) -> DecodeResult<i64> {
		Ok(self.read_i64::<BigEndian>()?)
	}

	fn read_int(&mut self) -> DecodeResult<i32> {
		Ok(self.read_i32::<BigEndian>()?)
	}

	fn read_short(&mut self) -> DecodeResult<i16> {
		Ok(self.read_i16::<BigEndian>()?)
	}

	fn read_unsigned_short(&mut self) -> DecodeResult<u16> {
		Ok(self.read_u16::<BigEndian>()?)
	}

	fn read_double(&mut self) -> DecodeResult<f64> {
		Ok(self.read_f64::<BigEndian>()?)
	}

	fn read_float(&mut self) -> DecodeResult<f32> {
		Ok(self.read_f32::<BigEndian>()?)
	}

	fn read_bool(&mut self) -> DecodeResult<bool> {
		Ok(matches!(self.read_u8()?, 1))
	}

	fn read_varint(&mut self) -> DecodeResult<i32> {
		let mut num_read = 0;
		let mut result = 0i32;
		let mut read;

		loop {
			read = self.read_byte()? as u8;
			let value = i32::from(read & 0b0111_1111);
			result |= value << (7 * num_read);

			num_read += 1;
			assert!((num_read <= 5), "VarInt is too big");

			if matches!(read & 0b1000_0000, 0) {
				break;
			}
		}

		Ok(result)
	}

	fn read_varlong(&mut self) -> DecodeResult<i64> {
		let mut num_read = 0;
		let mut result = 0i64;
		let mut read;
		loop {
			read = self.read_byte()? as u8;
			let value = i64::from(read & 0b0111_1111);
			result |= value << (7 * num_read);

			num_read += 1;
			assert!((num_read <= 5), "VarInt is too big");

			if matches!(read & 0b1000_0000, 0) {
				break;
			}
		}

		Ok(result)
	}

	fn read_string(&mut self) -> DecodeResult<String> {
		let length = self.read_varint()?;
		Ok(String::from_utf8(self.read_bytes(length as usize)?)?)
	}

	fn read_to_end(&mut self) -> DecodeResult<Vec<u8>> {
		let mut data = Vec::new();
		Read::read_to_end(self, &mut data)?;
		Ok(data)
	}

	fn read_position(&mut self) -> DecodeResult<(i32, i32, i32)> {
		let val = self.read_long()?;
		let x = val >> 38;
		let mut y = val & 0xFFF;
		if y >= 0x800 {
			y -= 0x1000;
		}

		let z = val << 26 >> 38;
		Ok((x as i32, y as i32, z as i32))
	}

	fn read_uuid(&mut self) -> DecodeResult<u128> {
		Ok(self.read_u128::<BigEndian>()?)
	}

	fn read_nbt_compound(&mut self) -> DecodeResult<Option<NbtCompound>> {
		let id = self.read_byte()? as u8;
		if matches!(id, 0) {
			return Ok(None);
		}

		let compound = match nbt::Value::from_reader(id, self)? {
			nbt::Value::Compound(compound) => Some(compound),
			_ => None,
		};

		Ok(compound)
	}

	fn read_player_property(&mut self) -> DecodeResult<PlayerProperty> {
		Ok(PlayerProperty {
			name: self.read_string()?,
			value: self.read_string()?,
			signature: {
				let has_signature = self.read_bool()?;
				if has_signature {
					Some(self.read_string()?)
				} else {
					None
				}
			},
		})
	}
}

// impl<T> PacketDecoderExt for Cursor<T> where T: AsRef<[u8]> {}
// impl PacketDecoderExt for TcpStream {}
impl<T> PacketDecoderExt for T where T: Read + Sized {}

pub trait PacketEncoderExt: Write {
	fn write_bytes(&mut self, value: &[u8]) {
		self.write_all(value).unwrap();
	}

	fn write_varint(&mut self, value: i32) {
		let _ = self.write_all(&PacketEncoder::varint(value));
	}

	fn write_varlong(&mut self, mut value: i64) {
		loop {
			let mut temp = (value & 0b1111_1111) as u8;
			value >>= 7;
			if !matches!(value, 0) {
				temp |= 0b1000_0000;
			}

			self.write_all(&[temp]).unwrap();
			if matches!(value, 0) {
				break;
			}
		}
	}

	fn write_byte(&mut self, value: i8) {
		self.write_bytes(&[value as u8]);
	}

	fn write_unsigned_byte(&mut self, value: u8) {
		self.write_bytes(&[value]);
	}

	fn write_short(&mut self, value: i16) {
		self.write_i16::<BigEndian>(value).unwrap();
	}

	fn write_unsigned_short(&mut self, value: u16) {
		self.write_u16::<BigEndian>(value).unwrap();
	}

	fn write_int(&mut self, value: i32) {
		self.write_i32::<BigEndian>(value).unwrap();
	}

	fn write_double(&mut self, value: f64) {
		self.write_f64::<BigEndian>(value).unwrap();
	}

	fn write_float(&mut self, value: f32) {
		self.write_f32::<BigEndian>(value).unwrap();
	}

	fn write_string(&mut self, n: usize, value: &str) {
		assert!(
			(value.len() <= n * 4 + 3),
			"tried to write a string longer than the max length"
		);

		self.write_varint(value.len() as i32);
		self.write_bytes(value.as_bytes());
	}

	fn write_identifier(&mut self, value: &str) {
		self.write_string(32767, value);
	}

	fn write_uuid(&mut self, value: u128) {
		self.write_u128::<BigEndian>(value).unwrap();
	}

	fn write_long(&mut self, value: i64) {
		self.write_i64::<BigEndian>(value).unwrap();
	}

	fn write_position(&mut self, x: i32, y: i32, z: i32) {
		let long = ((i64::from(x) & 0x3FF_FFFF) << 38)
			| ((i64::from(z) & 0x3FF_FFFF) << 12)
			| (i64::from(y) & 0xFFF);
		self.write_long(long);
	}

	fn write_bool(&mut self, value: bool) {
		self.write_u8(u8::from(value)).unwrap();
	}

	fn write_nbt<T: Serialize>(&mut self, nbt: &T) {
		let mut encoder = nbt::ser::Encoder::new(self, None, true);
		if let Err(err) = nbt.serialize(&mut encoder) {
			error!("there was an error encoding NBT in a packet: {err}");
		}
	}

	fn write_text_component(&mut self, value: &TextComponent)
	where
		Self: Sized,
	{
		if value.is_text_only() {
			let value = nbt::Value::String(value.text.clone());
			self.write_unsigned_byte(value.id());
			let _ = value.to_writer(self);
		} else {
			self.write_nbt(value);
		}
	}

	fn write_slot_data(&mut self, slot_data: Option<&SlotData>)
	where
		Self: Sized,
	{
		if let Some(slot) = slot_data {
			self.write_bool(true);
			self.write_varint(slot.item_id);
			self.write_byte(slot.item_count);
			if let Some(nbt) = &slot.nbt {
				self.write_nbt(nbt);
			} else {
				self.write_byte(0);
			}
		} else {
			self.write_bool(false);
		}
	}

	fn write_player_property(&mut self, player_property: &PlayerProperty)
	where
		Self: Sized,
	{
		self.write_identifier(&player_property.name);
		self.write_identifier(&player_property.value);
		self.write_bool(player_property.signature.is_some());
		if let Some(signature) = &player_property.signature {
			self.write_identifier(signature);
		}
	}
}

impl<T: Write> PacketEncoderExt for T {}

pub type DecodeResult<T, E = PacketDecodeError> = std::result::Result<T, E>;

pub fn read_packet<T: PacketDecoderExt>(
	reader: &mut T,
	compressed: &Arc<AtomicBool>,
	network_state: &mut NetworkState,
) -> DecodeResult<Box<dyn ServerBoundPacket>> {
	let length = reader.read_varint()?;
	let data = reader.read_bytes(length as usize)?;
	let mut cursor = Cursor::new(data);
	if compressed.load(Ordering::Relaxed) {
		read_compressed(&mut cursor, network_state)
	} else {
		read_decompressed(&mut cursor, network_state)
	}
}

fn read_compressed<T: PacketDecoderExt>(
	reader: &mut T,
	network_state: &mut NetworkState,
) -> DecodeResult<Box<dyn ServerBoundPacket>> {
	let decompressed_length = reader.read_varint()? as usize;
	let data = PacketDecoderExt::read_to_end(reader)?;

	if matches!(decompressed_length, 0) {
		read_decompressed(&mut Cursor::new(data), network_state)
	} else {
		let mut decompresser = ZlibDecoder::new(data.as_slice());
		let mut decompressed_data = Vec::with_capacity(decompressed_length);
		Read::read_to_end(&mut decompresser, &mut decompressed_data)?;
		read_decompressed(&mut Cursor::new(decompressed_data), network_state)
	}
}

fn read_decompressed<T: PacketDecoderExt>(
	reader: &mut T,
	state: &mut NetworkState,
) -> DecodeResult<Box<dyn ServerBoundPacket>> {
	let packet_id = reader.read_varint()?;
	let packet: Box<dyn ServerBoundPacket> = match *state {
		NetworkState::Handshaking if matches!(packet_id, 0x00) => {
			let handshake = SHandshake::decode(reader)?;
			match handshake.next_state {
				1 => *state = NetworkState::Status,
				2 => *state = NetworkState::Login,
				_ => {}
			}
			Box::new(handshake)
		}
		NetworkState::Status if matches!(packet_id, 0x00) => Box::new(SRequest::decode(reader)?),
		NetworkState::Status if matches!(packet_id, 0x01) => Box::new(SPing::decode(reader)?),
		NetworkState::Login if matches!(packet_id, 0x00) => Box::new(SLoginStart::decode(reader)?),
		NetworkState::Login if matches!(packet_id, 0x02) => {
			Box::new(SLoginPluginResponse::decode(reader)?)
		}
		NetworkState::Login if matches!(packet_id, 0x03) => {
			*state = NetworkState::Configuration;
			Box::new(SLoginAcknowledged::decode(reader)?)
		}
		NetworkState::Configuration if matches!(packet_id, 0x00) => {
			Box::new(SClientInformation::decode(reader)?)
		}
		NetworkState::Configuration if matches!(packet_id, 0x02) => {
			*state = NetworkState::Play;
			Box::new(SAcknowledgeFinishConfiguration::decode(reader)?)
		}
		_ => match packet_id {
			0x04 => Box::new(SChatCommand::decode(reader)?),
			0x05 => Box::new(SChatMessage::decode(reader)?),
			0x09 => Box::new(SClientInformation::decode(reader)?),
			0x0A => Box::new(SCommandSuggestionsRequest::decode(reader)?),
			0x10 => Box::new(SPluginMessage::decode(reader)?),
			0x15 => Box::new(SKeepAlive::decode(reader)?),
			0x17 => Box::new(SSetPlayerPosition::decode(reader)?),
			0x18 => Box::new(SSetPlayerPositionAndRotation::decode(reader)?),
			0x19 => Box::new(SPlayerRotation::decode(reader)?),
			0x1A => Box::new(SSetPlayerOnGround::decode(reader)?),
			0x20 => Box::new(SPlayerAbilities::decode(reader)?),
			0x21 => Box::new(SPlayerAction::decode(reader)?),
			0x22 => Box::new(SPlayerCommand::decode(reader)?),
			0x2C => Box::new(SSetHeldItem::decode(reader)?),
			0x2F => Box::new(SSetCreativeModeSlot::decode(reader)?),
			0x32 => Box::new(SUpdateSign::decode(reader)?),
			0x33 => Box::new(SSwingArm::decode(reader)?),
			0x35 => Box::new(SUseItemOn::decode(reader)?),
			_ => Box::new(SUnknown),
		},
	};

	trace!("received packet with id {packet_id:#02x}: {packet:?}");
	Ok(packet)
}
