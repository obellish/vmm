mod arflag;
mod cond;
pub mod cst;
mod div_modes;
mod hw_info;
mod instr;
mod prog;
mod reg;
mod val;

use std::borrow::Cow;

pub use self::{
	arflag::ArFlag,
	cond::If2Cond,
	div_modes::{DivByZeroMode, DivMode, DivOverflowMode, DivSignMode},
	hw_info::HwInfo,
	instr::{ExtInstr, Instr, InstrDecodingError},
	prog::{Program, ProgramDecodingError, ProgramWord},
	reg::Reg,
	val::{RegOrLit1, RegOrLit2},
};

pub trait ToLasm {
	fn to_lasm(&self) -> Cow<'static, str>;
}
