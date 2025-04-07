#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod nbt_util;
pub mod packets;

use std::{
	net::{Shutdown, TcpListener, TcpStream},
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
		mpsc,
	},
	thread,
};

use tracing::warn;

pub use self::nbt_util::NbtCompound;

#[derive(Debug)]
#[repr(transparent)]
pub struct PlayerPacketSender {
	stream: Option<TcpStream>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play
}
