mod config;

use vmm::cpu::Cpu;

pub use self::config::*;

#[derive(Debug, Clone, Copy)]
pub struct StoppedState {
	pub cycles: u128,
	pub addr: u32,
	pub ex: Option<ExWithMode>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExWithMode {
	pub raw: u32,
	pub sv_mode: bool,
	pub code: u8,
	pub associated: u16,
}
