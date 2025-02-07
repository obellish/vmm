#[cfg(feature = "serde")]
mod serde;

use alloc::{borrow::ToOwned, vec::Vec};
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	ops::{
		Add, AddAssign, BitAnd, BitOr, BitXor, Div, Index, IndexMut, Mul, Not, Rem, Sub, SubAssign,
	},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Length {
	pub(crate) index: usize,
}

impl Length {
	#[must_use]
	pub const fn new(index: usize) -> Self {
		Self { index }
	}

	#[must_use]
	pub const fn index(self) -> usize {
		self.index
	}
}

impl Add for Length {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		(self.index() + rhs.index()).into()
	}
}

impl Add<usize> for Length {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		(self.index() + rhs).into()
	}
}

impl Add<Length> for usize {
	type Output = Length;

	fn add(self, rhs: Length) -> Self::Output {
		(self + rhs.index()).into()
	}
}

impl AddAssign for Length {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl AddAssign<usize> for Length {
	fn add_assign(&mut self, rhs: usize) {
		*self = *self + rhs;
	}
}

impl Display for Length {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.index, f)
	}
}

impl From<usize> for Length {
	fn from(value: usize) -> Self {
		Self::new(value)
	}
}

impl From<Length> for usize {
	fn from(value: Length) -> Self {
		value.index()
	}
}

impl<T> Index<Length> for [T] {
	type Output = T;

	fn index(&self, index: Length) -> &Self::Output {
		&self[index.index()]
	}
}

impl<T> IndexMut<Length> for [T] {
	fn index_mut(&mut self, index: Length) -> &mut Self::Output {
		&mut self[index.index()]
	}
}

impl Not for Length {
	type Output = Self;

	fn not(self) -> Self::Output {
		(!self.index()).into()
	}
}

impl Sub for Length {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		(self.index() - rhs.index()).into()
	}
}

impl Sub<usize> for Length {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		(self.index() - rhs).into()
	}
}

impl Sub<Length> for usize {
	type Output = Length;

	fn sub(self, rhs: Length) -> Self::Output {
		(self - rhs.index()).into()
	}
}

impl SubAssign for Length {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl SubAssign<usize> for Length {
	fn sub_assign(&mut self, rhs: usize) {
		*self = *self - rhs;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SmallLength {
	Byte(u8),
	Word(u16),
	Double(u32),
	Quad(u64),
}

impl SmallLength {
	#[must_use]
	pub const fn index(self) -> usize {
		match self {
			Self::Byte(b) => b as usize,
			Self::Word(w) => w as usize,
			Self::Double(d) => d as usize,
			Self::Quad(q) => q as usize,
		}
	}

	#[must_use]
	pub fn from_bytes(bytes: &[u8]) -> Self {
		assert!(
			!bytes.is_empty(),
			"cannot convert empty bytes to small_length"
		);

		match bytes {
			[253, rest @ ..] => {
				assert_eq!(rest.len(), 2, "insufficient bytes for u16");
				Self::Word(u16::from_be_bytes([rest[0], rest[1]]))
			}
			[254, rest @ ..] => {
				assert_eq!(rest.len(), 4, "insufficient bytes for u32");
				Self::Double(u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]))
			}
			[255, rest @ ..] => {
				assert_eq!(rest.len(), 8, "insufficient bytes for u64");
				Self::Quad(u64::from_be_bytes([
					rest[0], rest[1], rest[2], rest[3], rest[4], rest[5], rest[6], rest[7],
				]))
			}
			[value, ..] => Self::Byte(*value),
			_ => panic!("unexpected byte format"),
		}
	}

	#[must_use]
	pub fn to_bytes(self) -> Vec<u8> {
		match self {
			Self::Byte(byte) => {
				if byte <= 252 {
					byte.to_be_bytes().to_vec()
				} else {
					let mut bytes = vec![253, 0, 0];
					let [a, b] = u16::from(byte).to_be_bytes();
					bytes[1] = a;
					bytes[2] = b;
					bytes
				}
			}
			Self::Word(word) => {
				let mut result = vec![253];
				result.extend_from_slice(&word.to_be_bytes());
				result
			}
			Self::Double(double) => {
				let mut result = vec![254];
				result.extend_from_slice(&double.to_be_bytes());
				result
			}
			Self::Quad(quad) => {
				let mut result = vec![255];
				result.extend_from_slice(&quad.to_be_bytes());
				result
			}
		}
	}
}

impl Default for SmallLength {
	fn default() -> Self {
		Self::Byte(0)
	}
}

impl Display for SmallLength {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::Byte(b) => Display::fmt(&b, f),
			Self::Word(w) => Display::fmt(&w, f),
			Self::Double(d) => Display::fmt(&d, f),
			Self::Quad(q) => Display::fmt(&q, f),
		}
	}
}

impl<T> Index<SmallLength> for [T] {
	type Output = T;

	fn index(&self, index: SmallLength) -> &Self::Output {
		&self[index.index()]
	}
}

impl<T> IndexMut<SmallLength> for [T] {
	fn index_mut(&mut self, index: SmallLength) -> &mut Self::Output {
		&mut self[index.index()]
	}
}
