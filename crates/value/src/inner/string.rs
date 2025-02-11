use alloc::string::String;
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
};

use crate::Logical;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct VmString {
	pub value: String,
}

impl Deref for VmString {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl DerefMut for VmString {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl Display for VmString {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self)
	}
}

impl From<String> for VmString {
	fn from(value: String) -> Self {
		Self { value }
	}
}

impl FromIterator<char> for VmString {
	fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
		Self {
			value: iter.into_iter().collect(),
		}
	}
}

impl Logical for VmString {
	fn and(&self, b: &Self) -> bool {
		!self.is_empty() && !b.is_empty()
	}

	fn or(&self, b: &Self) -> bool {
		!self.is_empty() || !b.is_empty()
	}

	fn xor(&self, b: &Self) -> bool {
		!self.is_empty() ^ !b.is_empty()
	}

	fn not(&self) -> bool {
		!self.is_empty()
	}

	fn bitwise_reverse(&self) -> Self {
		let s = self.chars().rev().collect::<String>();
		s.into()
	}
}
