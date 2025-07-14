use core::iter::FusedIterator;

use super::ValueType;
use crate::ir::{BasicBlockId, ValueId};

pub struct ValueIter<'a> {
	ty: &'a ValueType,
	index: usize,
}

impl<'a> ValueIter<'a> {
	pub(super) const fn new(ty: &'a ValueType) -> Self {
		Self { ty, index: 0 }
	}
}

impl FusedIterator for ValueIter<'_> {}

impl Iterator for ValueIter<'_> {
	type Item = ValueId;

	fn next(&mut self) -> Option<Self::Item> {
		let cur = self.index;
		self.index += 1;

		macro_rules! vec_use {
			($v:expr) => {
				if cur < $v.len() { Some($v[cur]) } else { None }
			};
		}

		macro_rules! field_use {
			($($field:expr),+) => {
				field_use!(@expand 0 $(,$field)+)
			};
			(@expand $index:expr) => {
				None
			};
			(@expand $index:expr, $head:expr $(,$tail:expr)*) => {
				if cur == $index {
					Some($head)
				} else {
					field_use!(@expand $index + 4 $(,$tail)*)
				}
			};
		}

		match self.ty {
			ValueType::Aggregate(v) => vec_use!(v.values()),
			ValueType::GlobalAlloc(v) => field_use!(v.init()),
			ValueType::Load(v) => field_use!(v.src()),
			ValueType::Store(v) => field_use!(v.value(), v.dest()),
			ValueType::GetPtr(v) => field_use!(v.src(), v.index()),
			ValueType::GetElementPtr(v) => field_use!(v.src(), v.index()),
			ValueType::Binary(v) => field_use!(v.lhs(), v.rhs()),
			ValueType::Branch(v) => {
				let tlen = v.true_args().len();
				if matches!(cur, 0) {
					Some(v.cond())
				} else if cur >= 1 && cur <= tlen {
					Some(v.true_args()[cur - 1])
				} else if cur > tlen && cur <= tlen + v.false_args().len() {
					Some(v.false_args()[cur - tlen - 1])
				} else {
					None
				}
			}
			ValueType::Jump(v) => vec_use!(v.args()),
			ValueType::Call(v) => vec_use!(v.args()),
			ValueType::Return(v) => match cur {
				0 => v.value(),
				_ => None,
			},
			_ => None,
		}
	}
}

pub struct BasicBlockIter<'a> {
	ty: &'a ValueType,
	index: usize,
}

impl<'a> BasicBlockIter<'a> {
	pub(super) const fn new(ty: &'a ValueType) -> Self {
		Self { ty, index: 0 }
	}
}

impl FusedIterator for BasicBlockIter<'_> {}

impl Iterator for BasicBlockIter<'_> {
	type Item = BasicBlockId;

	fn next(&mut self) -> Option<Self::Item> {
		let cur = self.index;
		self.index += 1;
		match self.ty {
			ValueType::Branch(b) => match cur {
				0 => Some(b.true_basic_block()),
				1 => Some(b.false_basic_block()),
				_ => None,
			},
			ValueType::Jump(j) => match cur {
				0 => Some(j.target()),
				_ => None,
			},
			_ => None,
		}
	}
}
