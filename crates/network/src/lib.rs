#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod nbt_util;
pub mod packets;

use std::{
	net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
		mpsc,
	},
	thread,
};

use tracing::warn;

pub use self::nbt_util::NbtCompound;
use self::packets::{PacketEncoder, PlayerProperty, read_packet, serverbound::ServerBoundPacket};

#[derive(Debug)]
#[repr(transparent)]
pub struct PlayerPacketSender {
	stream: Option<TcpStream>,
}

impl PlayerPacketSender {
	pub fn new(conn: &PlayerConn) -> Self {
		let stream = conn.client.stream.try_clone().ok();
		if stream.is_none() {
			warn!("creating PacketPlayerSender with dead stream");
		}

		Self { stream }
	}

	pub fn write_compressed(&self, data: &PacketEncoder) {
		if let Some(stream) = &self.stream {
			let _ = data.write_compressed(stream);
		}
	}
}

pub struct HandshakingConn {
	client: NetworkClient,
	pub username: Option<String>,
	pub uuid: Option<u128>,
	pub forwarding_message_id: Option<i32>,
	pub properties: Vec<PlayerProperty>,
}

impl HandshakingConn {
	pub fn send_packet(&self, data: &PacketEncoder) {
		self.client.send_packet(data);
	}

	#[must_use]
	pub fn receive_packets(&self) -> Vec<Box<dyn ServerBoundPacket>> {
		self.client.receive_packets(&mut true)
	}

	pub fn set_compressed(&self, compressed: bool) {
		self.client.compressed.store(compressed, Ordering::Relaxed);
	}

	pub fn close_connection(&self) {
		self.client.close_connection();
	}
}

pub struct PlayerConn {
	client: NetworkClient,
	alive: bool,
}

impl PlayerConn {
	pub fn send_packet(&self, data: &PacketEncoder) {
		self.client.send_packet(data);
	}

	pub fn receive_packets(&mut self) -> Vec<Box<dyn ServerBoundPacket>> {
		self.client.receive_packets(&mut self.alive)
	}

	#[must_use]
	pub const fn alive(&self) -> bool {
		self.alive
	}

	pub fn close_connection(&mut self) {
		self.alive = false;
		self.client.close_connection();
	}
}

impl From<HandshakingConn> for PlayerConn {
	fn from(value: HandshakingConn) -> Self {
		Self {
			client: value.client,
			alive: true,
		}
	}
}

pub struct NetworkClient {
	pub id: u32,
	stream: TcpStream,
	packets: mpsc::Receiver<Box<dyn ServerBoundPacket>>,
	compressed: Arc<AtomicBool>,
}

impl NetworkClient {
	fn listen(
		mut stream: TcpStream,
		sender: mpsc::Sender<Box<dyn ServerBoundPacket>>,
		compressed: Arc<AtomicBool>,
	) {
		let mut state = NetworkState::Handshaking;
		loop {
			let Ok(packet) = read_packet(&mut stream, &compressed, &mut state) else {
				break;
			};

			if sender.send(packet).is_err() {
				break;
			}
		}
	}

	pub fn receive_packets(&self, alive: &mut bool) -> Vec<Box<dyn ServerBoundPacket>> {
		let mut packets = Vec::new();
		loop {
			let packet = self.packets.try_recv();
			match packet {
				Ok(packet) => packets.push(packet),
				Err(mpsc::TryRecvError::Empty) => break packets,
				_ => {
					*alive = false;
					break packets;
				}
			}
		}
	}

	pub fn send_packet(&self, data: &PacketEncoder) {
		if self.compressed.load(Ordering::Relaxed) {
			let _ = data.write_compressed(&self.stream);
		} else {
			let _ = data.write_uncompressed(&self.stream);
		}
	}

	pub fn close_connection(&self) {
		let _ = self.stream.shutdown(Shutdown::Both);
	}
}

pub struct NetworkServer {
	client_receiver: mpsc::Receiver<NetworkClient>,
	pub handshaking_clients: Vec<HandshakingConn>,
}

impl NetworkServer {
	fn listen(bind_address: impl ToSocketAddrs, sender: mpsc::Sender<NetworkClient>) {
		let listener = TcpListener::bind(bind_address).unwrap();

		for (index, stream) in listener.incoming().enumerate() {
			let stream = stream.unwrap();
			let (packet_sender, packet_receiver) = mpsc::channel();
			let compressed = Arc::new(AtomicBool::new(false));
			let client_stream = stream.try_clone().unwrap();
			let client_compressed = compressed.clone();
			thread::spawn(move || {
				NetworkClient::listen(client_stream, packet_sender, client_compressed);
			});

			sender
				.send(NetworkClient {
					id: index as u32,
					stream,
					packets: packet_receiver,
					compressed,
				})
				.unwrap();
		}
	}

	pub fn new<A>(bind_address: A) -> Self
	where
		A: Send + ToSocketAddrs + 'static,
	{
		let (sender, receiver) = mpsc::channel();
		thread::spawn(move || Self::listen(bind_address, sender));
		Self {
			client_receiver: receiver,
			handshaking_clients: Vec::new(),
		}
	}

	pub fn update(&mut self) {
		loop {
			match self.client_receiver.try_recv() {
				Ok(client) => self.handshaking_clients.push(HandshakingConn {
					client,
					username: None,
					uuid: None,
					forwarding_message_id: None,
					properties: Vec::new(),
				}),
				Err(mpsc::TryRecvError::Empty) => break,
				Err(mpsc::TryRecvError::Disconnected) => {
					panic!("client receiver channel disconnected")
				}
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
	Handshaking,
	Status,
	Login,
	Configuration,
	Play,
}
