use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
	slice,
};

use serde::{Deserialize, Serialize};

use crate::Instruction;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Program {
	Raw(Vec<Instruction>),
	Optimized(Box<[Instruction]>),
}

impl Program {
	#[must_use]
	pub const fn is_optimized(&self) -> bool {
		matches!(self, Self::Optimized(_))
	}

	pub fn as_raw(&mut self) -> &mut Vec<Instruction> {
		match self {
			Self::Raw(ops) => ops,
			Self::Optimized(ops) => {
				*self = Self::Raw(ops.to_vec());

				match self {
					Self::Raw(ops) => ops,
					Self::Optimized(_) => unreachable!(),
				}
			}
		}
	}

	#[must_use]
	pub fn needs_input(&self) -> bool {
		self.iter().any(Instruction::needs_input)
	}
}

impl Debug for Program {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_list().entries(&**self).finish()
	}
}

impl Default for Program {
	fn default() -> Self {
		Self::Raw(Vec::new())
	}
}

impl Deref for Program {
	type Target = [Instruction];

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Raw(ops) => ops,
			Self::Optimized(ops) => ops,
		}
	}
}

impl DerefMut for Program {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Raw(ops) => ops,
			Self::Optimized(ops) => ops,
		}
	}
}

impl FromIterator<Instruction> for Program {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = Instruction>,
	{
		Self::Raw(iter.into_iter().collect())
	}
}

impl<'a> IntoIterator for &'a Program {
	type IntoIter = slice::Iter<'a, Instruction>;
	type Item = &'a Instruction;

	fn into_iter(self) -> Self::IntoIter {
		(**self).iter()
	}
}
