#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
	slice,
};

use serde::{Deserialize, Serialize};
use vmm_ir::Instruction;
use vmm_utils::HeapSize;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Program {
	Raw(Vec<Instruction>),
	Finalized(Box<[Instruction]>),
}

impl Program {
	#[must_use]
	pub const fn is_finalized(&self) -> bool {
		matches!(self, Self::Finalized(_))
	}

	#[inline]
	pub fn as_raw(&mut self) -> &mut Vec<Instruction> {
		match self {
			Self::Raw(ops) => ops,
			Self::Finalized(ops) => {
				*self = Self::Raw(ops.to_vec());

				match self {
					Self::Raw(ops) => ops,
					Self::Finalized(_) => unreachable!(),
				}
			}
		}
	}

	#[inline]
	pub fn rough_estimate(&self) -> usize {
		self.iter().map(Instruction::rough_estimate).sum()
	}

	#[inline]
	pub fn raw_rough_estimate(&self) -> usize {
		self.iter().map(Instruction::raw_rough_estimate).sum()
	}

	#[inline]
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
			Self::Finalized(ops) => ops,
		}
	}
}

impl DerefMut for Program {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Raw(ops) => ops,
			Self::Finalized(ops) => ops,
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

impl HeapSize for Program {
	fn heap_size(&self) -> usize {
		match self {
			Self::Raw(v) => v.heap_size(),
			Self::Finalized(b) => b.heap_size(),
		}
	}
}

impl<'a> IntoIterator for &'a Program {
	type IntoIter = slice::Iter<'a, Instruction>;
	type Item = &'a Instruction;

	fn into_iter(self) -> Self::IntoIter {
		(**self).iter()
	}
}

impl<'a> IntoIterator for &'a mut Program {
	type IntoIter = slice::IterMut<'a, Instruction>;
	type Item = &'a mut Instruction;

	fn into_iter(self) -> Self::IntoIter {
		(**self).iter_mut()
	}
}
