use std::borrow::Cow;

use crate::asm::{Instr, Program, ProgramWord, Reg, ToLasm};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtInstr {
	SetReg(Reg, u32),
	ReadAddr(u32),
	ReadAddrTo(Reg, u32),
	WriteAddr(u32, Reg),
	WriteAddrLit(u32, u32),
}

impl ExtInstr {
	#[must_use]
	pub fn to_instr(self) -> Vec<Instr> {
		match self {
			Self::SetReg(reg, value) => vec![
				Instr::Cpy(reg, ((value >> 16) as u16).into()),
				Instr::Shl(reg, 16u8.into()),
				Instr::Add(reg, (value as u16).into()),
			],
			Self::ReadAddr(addr) => vec![
				Instr::Cpy(Reg::Avr, ((addr >> 16) as u16).into()),
				Instr::Shl(Reg::Avr, 16u8.into()),
				Instr::Add(Reg::Avr, (addr as u16).into()),
				Instr::Lea(Reg::Avr.into(), 0u8.into(), 0u8.into()),
			],
			Self::ReadAddrTo(reg, addr) => vec![
				Instr::Cpy(Reg::Avr, ((addr >> 16) as u16).into()),
				Instr::Shl(Reg::Avr, 16u8.into()),
				Instr::Add(Reg::Avr, (addr as u16).into()),
				Instr::Lea(Reg::Avr.into(), 0u8.into(), 0u8.into()),
				Instr::Cpy(reg, Reg::Avr.into()),
			],
			Self::WriteAddr(addr, reg_value) => vec![
				Instr::Cpy(Reg::Rr0, ((addr >> 16) as u16).into()),
				Instr::Shl(Reg::Rr0, 16u8.into()),
				Instr::Add(Reg::Rr0, (addr as u16).into()),
				Instr::Cpy(Reg::Avr, reg_value.into()),
				Instr::Wea(Reg::Rr0.into(), 0u8.into(), 0u8.into()),
			],
			Self::WriteAddrLit(addr, value) => vec![
				Instr::Cpy(Reg::Rr0, ((addr >> 16) as u16).into()),
				Instr::Shl(Reg::Rr0, 16u8.into()),
				Instr::Add(Reg::Rr0, (addr as u16).into()),
				Instr::Cpy(Reg::Avr, ((value >> 16) as u16).into()),
				Instr::Shl(Reg::Avr, 16u8.into()),
				Instr::Add(Reg::Avr, (value as u16).into()),
				Instr::Wea(Reg::Rr0.into(), 0u8.into(), 0u8.into()),
			],
		}
	}

	pub fn to_prog_words(self) -> Vec<ProgramWord> {
		self.to_instr().into_iter().map(ProgramWord::from).collect()
	}

	#[must_use]
	pub fn encode_words(self) -> Vec<u32> {
		Program::from_iter(self.to_instr()).encode_words()
	}

	#[must_use]
	pub fn encode(self) -> Vec<u8> {
		Program::from_iter(self.to_instr()).encode()
	}
}

impl ToLasm for ExtInstr {
	fn to_lasm(&self) -> Cow<'static, str> {
		Cow::Owned(Program::from_iter(self.to_instr()).to_lasm(false))
	}
}
