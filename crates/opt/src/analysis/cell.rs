use std::{
	cmp::Ordering,
	num::NonZeroU8,
	ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
	value: CellValue,
	state: CellState,
}

impl Cell {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			value: CellValue::Unknown,
			state: CellState::empty(),
		}
	}

	#[must_use]
	pub const fn value(self) -> CellValue {
		self.value
	}

	pub const fn value_mut(&mut self) -> &mut CellValue {
		&mut self.value
	}

	#[must_use]
	pub const fn state(self) -> CellState {
		self.state
	}

	pub const fn state_mut(&mut self) -> &mut CellState {
		&mut self.state
	}
}

impl BitAnd<CellState> for Cell {
	type Output = Self;

	fn bitand(self, rhs: CellState) -> Self::Output {
		Self {
			value: self.value,
			state: self.state.intersection(rhs),
		}
	}
}

impl BitAndAssign<CellState> for Cell {
	fn bitand_assign(&mut self, rhs: CellState) {
		*self.state_mut() = CellState::from_bits_retain(self.state().bits()).intersection(rhs);
	}
}

impl BitOr<CellState> for Cell {
	type Output = Self;

	fn bitor(self, rhs: CellState) -> Self::Output {
		Self {
			value: self.value,
			state: self.state.union(rhs),
		}
	}
}

impl BitOrAssign<CellState> for Cell {
	fn bitor_assign(&mut self, rhs: CellState) {
		self.state_mut().insert(rhs);
	}
}

impl BitXor<CellState> for Cell {
	type Output = Self;

	fn bitxor(self, rhs: CellState) -> Self::Output {
		Self {
			value: self.value,
			state: self.state.symmetric_difference(rhs),
		}
	}
}

impl BitXorAssign<CellState> for Cell {
	fn bitxor_assign(&mut self, rhs: CellState) {
		self.state_mut().toggle(rhs);
	}
}

impl Extend<CellState> for Cell {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = CellState>,
	{
		self.state_mut().extend(iter);
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct CellState(u8);

bitflags! {
	impl CellState: u8 {
		const TOUCHED = 0b0000_0001;
		const IN_LOOP = 0b0000_0010;
		const WRITTEN = 0b0000_0100;
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellValue {
	#[default]
	Unknown,
	Zero,
	Known(NonZeroU8),
}

impl From<u8> for CellValue {
	fn from(value: u8) -> Self {
		match value {
			0 => Self::Zero,
			n => Self::Known(unsafe { NonZeroU8::new_unchecked(n) }),
		}
	}
}

impl Ord for CellValue {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(Self::Unknown, Self::Unknown) | (Self::Zero, Self::Zero) => Ordering::Equal,
			(Self::Unknown, _) | (Self::Zero, Self::Known(_)) => Ordering::Less,
			(Self::Zero, Self::Unknown) | (Self::Known(_), Self::Unknown | Self::Zero) => {
				Ordering::Greater
			}
			(Self::Known(l), Self::Known(r)) => l.cmp(r),
		}
	}
}

impl PartialEq<u8> for CellValue {
	fn eq(&self, other: &u8) -> bool {
		match self {
			Self::Unknown => false,
			Self::Zero => matches!(*other, 0),
			Self::Known(v) => v.get() == *other,
		}
	}
}

impl PartialEq<CellValue> for u8 {
	fn eq(&self, other: &CellValue) -> bool {
		*other == *self
	}
}

impl PartialOrd for CellValue {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialOrd<u8> for CellValue {
	fn partial_cmp(&self, other: &u8) -> Option<Ordering> {
		Some(self.cmp(&(*other).into()))
	}
}

impl PartialOrd<CellValue> for u8 {
	fn partial_cmp(&self, other: &CellValue) -> Option<Ordering> {
		Some(CellValue::from(*self).cmp(other))
	}
}
