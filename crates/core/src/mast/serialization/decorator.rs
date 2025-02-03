use alloc::vec::Vec;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use super::{
	DecoratorDataOffset,
	string_table::{StringTable, StringTableBuilder},
};
use crate::{
	AssemblyOp, DebugOptions, Decorator,
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, SliceReader,
	},
};

#[derive(Debug)]
pub struct DecoratorInfo {
	variant: EncodedDecoratorVariant,
	decorator_data_offset: DecoratorDataOffset,
}

impl DecoratorInfo {
	pub fn from_decorator(
		decorator: &Decorator,
		decorator_data_offset: DecoratorDataOffset,
	) -> Self {
		let variant = EncodedDecoratorVariant::from(decorator);

		Self {
			variant,
			decorator_data_offset,
		}
	}

	pub fn try_into_decorator(
		&self,
		string_table: &StringTable,
		decorator_data: &[u8],
	) -> Result<Decorator, DeserializationError> {
		let mut data_reader =
			SliceReader::new(&decorator_data[self.decorator_data_offset as usize..]);

		match self.variant {
			EncodedDecoratorVariant::AssemblyOp => {
				let num_cycles = data_reader.read_u8()?;
				let should_break = data_reader.read_bool()?;

				let location = if data_reader.read_bool()? {
					let str_index_in_table = data_reader.read_usize()?;
					let path = string_table.read_arc_string(str_index_in_table)?;
					let start = data_reader.read_u32()?;
					let end = data_reader.read_u32()?;
					Some(crate::debuginfo::Location {
						path,
						start: start.into(),
						end: end.into(),
					})
				} else {
					None
				};

				let context_name = {
					let str_index_in_table = data_reader.read_usize()?;
					string_table.read_string(str_index_in_table)?
				};

				let op = {
					let str_index_in_table = data_reader.read_usize()?;
					string_table.read_string(str_index_in_table)?
				};

				Ok(Decorator::AsmOp(AssemblyOp::new(
					location,
					context_name,
					op,
					num_cycles,
					should_break,
				)))
			}
			EncodedDecoratorVariant::DebugOptionsStackAll => {
				Ok(Decorator::Debug(DebugOptions::StackAll))
			}
			EncodedDecoratorVariant::DebugOptionsStackTop => {
				let value = data_reader.read_u8()?;

				Ok(Decorator::Debug(DebugOptions::StackTop(value)))
			}
			EncodedDecoratorVariant::DebugOptionsMemAll => {
				Ok(Decorator::Debug(DebugOptions::MemAll))
			}
			EncodedDecoratorVariant::DebugOptionsMemInterval => {
				let start = data_reader.read_u32()?;
				let end = data_reader.read_u32()?;

				Ok(Decorator::Debug(DebugOptions::MemInterval(start, end)))
			}
			EncodedDecoratorVariant::DebugOptionsLocalInterval => {
				let start = data_reader.read_u16()?;
				let second = data_reader.read_u16()?;
				let end = data_reader.read_u16()?;

				Ok(Decorator::Debug(DebugOptions::LocalInterval(
					start, second, end,
				)))
			}
			EncodedDecoratorVariant::Trace => {
				let value = data_reader.read_u32()?;

				Ok(Decorator::Trace(value))
			}
		}
	}
}

impl Deserializable for DecoratorInfo {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let variant = source.read()?;
		let decorator_data_offset = source.read()?;

		Ok(Self {
			variant,
			decorator_data_offset,
		})
	}
}

impl Serializable for DecoratorInfo {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let Self {
			variant,
			decorator_data_offset,
		} = self;

		variant.write_into(target);
		decorator_data_offset.write_into(target);
	}
}

#[derive(Debug, Default)]
pub struct DecoratorDataBuilder {
	decorator_data: Vec<u8>,
	decorator_infos: Vec<DecoratorInfo>,
	string_table_builder: StringTableBuilder,
}

impl DecoratorDataBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_decorator(&mut self, decorator: &Decorator) {
		let decorator_data_offset = self.encode_decorator_data(decorator).unwrap_or_default();
		self.decorator_infos.push(DecoratorInfo::from_decorator(
			decorator,
			decorator_data_offset,
		));
	}

	pub fn encode_decorator_data(&mut self, decorator: &Decorator) -> Option<DecoratorDataOffset> {
		let data_offset = self.decorator_data.len() as DecoratorDataOffset;

		match decorator {
			Decorator::AsmOp(assembly_op) => {
				self.decorator_data.push(assembly_op.num_cycles());
				self.decorator_data.write_bool(assembly_op.should_break());

				let loc = assembly_op.location();
				self.decorator_data.write_bool(loc.is_some());
				if let Some(loc) = loc {
					let str_offset = self.string_table_builder.add_string(&loc.path);
					self.decorator_data.write_usize(str_offset);
					self.decorator_data.write_u32(loc.start.to_u32());
					self.decorator_data.write_u32(loc.end.to_u32());
				}

				{
					let str_offset = self
						.string_table_builder
						.add_string(assembly_op.context_name());
					self.decorator_data.write_usize(str_offset);
				}

				{
					let str_index_in_table = self.string_table_builder.add_string(assembly_op.op());
					self.decorator_data.write_usize(str_index_in_table);
				}

				Some(data_offset)
			}
			Decorator::Debug(DebugOptions::StackTop(value)) => {
				self.decorator_data.push(*value);
				Some(data_offset)
			}
			Decorator::Debug(DebugOptions::MemInterval(start, end)) => {
				self.decorator_data.extend(start.to_le_bytes());
				self.decorator_data.extend(end.to_le_bytes());

				Some(data_offset)
			}
			Decorator::Debug(DebugOptions::LocalInterval(start, second, end)) => {
				self.decorator_data.extend(start.to_le_bytes());
				self.decorator_data.extend(second.to_le_bytes());
				self.decorator_data.extend(end.to_le_bytes());

				Some(data_offset)
			}
			Decorator::Debug(DebugOptions::StackAll | DebugOptions::MemAll) => None,
			Decorator::Trace(value) => {
				self.decorator_data.extend(value.to_le_bytes());

				Some(data_offset)
			}
		}
	}

	pub fn finalize(self) -> (Vec<u8>, Vec<DecoratorInfo>, StringTable) {
		(
			self.decorator_data,
			self.decorator_infos,
			self.string_table_builder.into_table(),
		)
	}
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum EncodedDecoratorVariant {
	AssemblyOp,
	DebugOptionsStackAll,
	DebugOptionsStackTop,
	DebugOptionsMemAll,
	DebugOptionsMemInterval,
	DebugOptionsLocalInterval,
	Trace,
}

impl EncodedDecoratorVariant {
	pub fn discriminant(&self) -> u8 {
		self.to_u8().expect("guaranteed to fit in a u8")
	}

	pub fn from_distriminant(discriminant: u8) -> Option<Self> {
		Self::from_u8(discriminant)
	}
}

impl Deserializable for EncodedDecoratorVariant {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let discriminant = source.read_u8()?;

		Self::from_distriminant(discriminant).ok_or_else(|| {
			DeserializationError::InvalidValue(format!(
				"invalid decorator discriminant: {discriminant}"
			))
		})
	}
}

impl From<&Decorator> for EncodedDecoratorVariant {
	fn from(value: &Decorator) -> Self {
		match value {
			Decorator::AsmOp(_) => Self::AssemblyOp,
			Decorator::Debug(DebugOptions::StackAll) => Self::DebugOptionsStackAll,
			Decorator::Debug(DebugOptions::StackTop(_)) => Self::DebugOptionsStackTop,
			Decorator::Debug(DebugOptions::MemAll) => Self::DebugOptionsMemAll,
			Decorator::Debug(DebugOptions::MemInterval(..)) => Self::DebugOptionsMemInterval,
			Decorator::Debug(DebugOptions::LocalInterval(..)) => Self::DebugOptionsLocalInterval,
			Decorator::Trace(_) => Self::Trace,
		}
	}
}

impl Serializable for EncodedDecoratorVariant {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.discriminant().write_into(target);
	}
}
