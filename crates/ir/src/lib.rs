#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	num::NonZeroU8,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	IncVal {
		value: i8,
		offset: Option<Offset>,
	},
	SetVal {
		value: Option<NonZeroU8>,
		offset: Option<Offset>,
	},
	MoveVal {
		offset: Offset,
		factor: u8,
	},
	FetchVal {
		offset: Offset,
		factor: u8,
	},
	MovePtr(Offset),
	FindZero(isize),
	Read,
	Write,
	DynamicLoop(Vec<Self>),
}

impl Instruction {
	#[must_use]
	pub const fn inc_val(v: i8) -> Self {
		Self::IncVal {
			value: v,
			offset: None,
		}
	}

	#[must_use]
	pub const fn inc_val_relative(v: i8, offset: isize) -> Self {
		Self::IncVal {
			value: v,
			offset: Some(Offset::Relative(offset)),
		}
	}

	#[must_use]
	pub const fn set_val(v: u8) -> Self {
		Self::SetVal {
			value: NonZeroU8::new(v),
			offset: None,
		}
	}

	#[must_use]
	pub const fn set_val_relative(v: u8, offset: isize) -> Self {
		Self::SetVal {
			value: NonZeroU8::new(v),
			offset: Some(Offset::Relative(offset)),
		}
	}

	#[must_use]
	pub const fn clear_val() -> Self {
		Self::SetVal {
			value: None,
			offset: None,
		}
	}

	#[must_use]
	pub const fn clear_val_relative(offset: isize) -> Self {
		Self::SetVal {
			value: None,
			offset: Some(Offset::Relative(offset)),
		}
	}

	#[must_use]
	pub const fn move_val_by(offset: isize, factor: u8) -> Self {
		Self::MoveVal {
			offset: Offset::Relative(offset),
			factor,
		}
	}

	#[must_use]
	pub const fn move_val_to(index: usize, factor: u8) -> Self {
		Self::MoveVal {
			offset: Offset::Absolute(index),
			factor,
		}
	}

	#[must_use]
	pub const fn move_ptr_by(offset: isize) -> Self {
		Self::MovePtr(Offset::Relative(offset))
	}

	#[must_use]
	pub const fn move_ptr_to(index: usize) -> Self {
		Self::MovePtr(Offset::Absolute(index))
	}

	#[must_use]
	pub const fn find_zero(jump_by: isize) -> Self {
		Self::FindZero(jump_by)
	}

	#[must_use]
	pub const fn read() -> Self {
		Self::Read
	}

	#[must_use]
	pub const fn write() -> Self {
		Self::Write
	}

	pub fn dynamic_loop(instructions: impl IntoIterator<Item = Self>) -> Self {
		Self::DynamicLoop(instructions.into_iter().collect())
	}

	pub fn needs_input(&self) -> bool {
		match self {
			Self::Read => true,
			Self::DynamicLoop(instrs) => instrs.iter().any(Self::needs_input),
			_ => false,
		}
	}

	pub fn has_side_effect(&self) -> bool {
		match self {
			Self::Read | Self::Write => true,
			Self::DynamicLoop(instrs) => instrs.iter().any(Self::has_side_effect),
			_ => false,
		}
	}

	#[must_use]
	pub const fn is_inc_val(&self) -> bool {
		matches!(self, Self::IncVal {value, ..} if *value > 0)
	}

	#[must_use]
	pub const fn is_dec_val(&self) -> bool {
		matches!(self, Self::IncVal {value, ..} if *value < 0 )
	}

	#[must_use]
	pub const fn is_set_val(&self) -> bool {
		matches!(self, Self::SetVal { .. })
	}

	#[must_use]
	pub const fn is_clear_val(&self) -> bool {
		matches!(self, Self::SetVal { .. })
	}

	#[must_use]
	pub const fn is_move_val(&self) -> bool {
		matches!(self, Self::MoveVal { .. })
	}

	#[must_use]
	pub const fn is_loop(&self) -> bool {
		matches!(self, Self::DynamicLoop(_))
	}

	#[must_use]
	pub fn is_empty_loop(&self) -> bool {
		matches!(self, Self::DynamicLoop(l) if l.is_empty())
	}

	#[must_use]
	pub const fn is_io(&self) -> bool {
		self.is_read() || self.is_write()
	}

	#[must_use]
	pub const fn is_read(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub const fn is_write(&self) -> bool {
		matches!(self, Self::Write)
	}

	pub fn rough_estimate(&self) -> usize {
		match self {
			Self::DynamicLoop(l) => l.iter().map(Self::rough_estimate).sum::<usize>() + 2,
			_ => 1,
		}
	}

	pub fn ptr_movement_of<'a>(iter: impl IntoIterator<Item = &'a Self>) -> Option<isize> {
		let mut movement = 0;

		for instr in iter {
			movement += instr.ptr_movement()?;
		}

		Some(movement)
	}

	#[must_use]
	pub fn ptr_movement(&self) -> Option<isize> {
		match self {
			Self::MoveVal { .. }
			| Self::IncVal { .. }
			| Self::SetVal { .. }
			| Self::Read
			| Self::Write => Some(0),
			Self::MovePtr(Offset::Relative(i)) => Some(*i),
			Self::DynamicLoop(instrs) => {
				let mut sum = 0;

				for instr in instrs {
					sum += instr.ptr_movement()?;
				}

				Some(sum)
			}
			_ => None,
		}
	}

	#[must_use]
	pub fn might_move_ptr(&self) -> bool {
		self.ptr_movement()
			.is_none_or(|offset| !matches!(offset, 0))
	}
}

impl Display for Instruction {
	#[allow(unreachable_patterns)] // For when we add more instructions.
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::IncVal {
				value: i,
				offset: None,
			} => {
				let c = if *i < 0 { '-' } else { '+' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::MovePtr(Offset::Relative(i)) => {
				let c = if *i < 0 { '<' } else { '>' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::SetVal {
				value: i,
				offset: None,
			} => {
				f.write_str("[-]")?;
				if let Some(i) = i {
					for _ in 0..i.get() {
						f.write_char('+')?;
					}
				}
			}
			Self::FindZero(i) => {
				f.write_char('[')?;
				let c = if *i < 0 { '<' } else { '>' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
				f.write_char(']')?;
			}
			Self::Read => f.write_char(',')?,
			Self::Write => f.write_char('.')?,
			Self::DynamicLoop(instrs) => {
				f.write_char('[')?;
				display_loop(instrs, f)?;
				f.write_char(']')?;
			}
			Self::MoveVal {
				offset: Offset::Relative(offset),
				factor: multiplier,
			} => {
				f.write_char('[')?;

				f.write_char('-')?;

				Display::fmt(&Self::MovePtr(Offset::Relative(*offset)), f)?;

				for _ in 0..*multiplier {
					f.write_char('+')?;
				}

				Display::fmt(&Self::MovePtr(Offset::Relative(-offset)), f)?;

				f.write_char(']')?;
			}
			Self::IncVal {
				value,
				offset: Some(Offset::Relative(offset)),
			} => {
				Display::fmt(&Self::MovePtr(Offset::Relative(*offset)), f)?;
				Display::fmt(
					&Self::IncVal {
						value: *value,
						offset: None,
					},
					f,
				)?;

				Display::fmt(&Self::MovePtr(Offset::Relative(-offset)), f)?;
			}
			Self::SetVal {
				value,
				offset: Some(Offset::Relative(offset)),
			} => {
				Display::fmt(&Self::MovePtr((*offset).into()), f)?;
				Display::fmt(
					&Self::SetVal {
						value: *value,
						offset: None,
					},
					f,
				)?;
				Display::fmt(&Self::MovePtr((-offset).into()), f)?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

fn display_loop(i: &[Instruction], f: &mut Formatter<'_>) -> FmtResult {
	for instr in i {
		Display::fmt(&instr, f)?;
	}

	Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Offset {
	Relative(isize),
	Absolute(usize),
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Absolute(v) => Display::fmt(&v, f)?,
			Self::Relative(v) => Display::fmt(&v, f)?,
		}

		Ok(())
	}
}

impl From<isize> for Offset {
	fn from(value: isize) -> Self {
		Self::Relative(value)
	}
}

impl From<usize> for Offset {
	fn from(value: usize) -> Self {
		Self::Absolute(value)
	}
}
