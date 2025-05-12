use std::{
	num::Wrapping,
	ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
	value: Wrapping<u8>,
	state: CellState,
}

impl Cell {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			value: Wrapping(0),
			state: CellState::empty(),
		}
	}

	#[must_use]
	pub const fn value(&self) -> &Wrapping<u8> {
		&self.value
	}

	pub const fn value_mut(&mut self) -> &mut Wrapping<u8> {
		&mut self.value
	}

	#[must_use]
	pub const fn state(&self) -> &CellState {
		&self.state
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
