use std::mem;

#[derive(Debug)]
pub struct Registers {
	pub a: [u32; 8],
	pub c: [u32; 2],
	pub ac: [u32; 3],
	pub rr: [u32; 8],
	pub avr: u32,
	pub af: u32,
	pub pc: u32,
	pub ssp: u32,
	pub usp: u32,
	pub et: u32,
	pub era: u32,
	pub ev: u32,
	pub mtt: u32,
	pub pda: u32,
	pub smt: u32,
}

impl Registers {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			a: [0; 8],
			c: [0; 2],
			ac: [0; 3],
			rr: [0; 8],
			avr: 0,
			af: 0,
			pc: 0,
			ssp: 0,
			usp: 0,
			et: 0,
			era: 0,
			ev: 0,
			mtt: 0,
			pda: 0,
			smt: 0,
		}
	}

	pub fn reset(&mut self) {
		mem::take(self);
	}
}

impl Default for Registers {
	fn default() -> Self {
		Self::new()
	}
}
