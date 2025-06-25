use alloc::{vec, vec::Vec};
use core::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Bytes {
	Single(u8),
	Many(Vec<u8>),
}

impl Bytes {
	#[must_use]
	pub fn into_vec(self) -> Vec<u8> {
		match self {
			Self::Single(b) => Vec::from([b]),
			Self::Many(b) => b,
		}
	}

	#[must_use]
	pub fn to_many(&self) -> Self {
		match self {
			Self::Many(v) => Self::Many(v.clone()),
			Self::Single(b) => Self::Many(vec![*b]),
		}
	}

	pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		match self {
			Self::Many(b) => b,
			Self::Single(_) => {
				*self = self.to_many();
				match self {
					Self::Many(b) => b,
					Self::Single(_) => unreachable!(),
				}
			}
		}
	}

	pub fn shrink(&mut self) {
		match self {
			Self::Many(b) if matches!(b.len(), 1) => *self = Self::Single(b[0]),
			_ => {}
		}
	}
}

impl Add for Bytes {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Single(l), Self::Single(r)) => Self::Many(vec![l, r]),
			(Self::Single(l), Self::Many(mut r)) => Self::Many({
				r.insert(0, l);
				r
			}),
			(Self::Many(mut l), Self::Many(r)) => Self::Many({
				l.extend(r);
				l
			}),
			(Self::Many(mut l), Self::Single(r)) => Self::Many({
				l.push(r);
				l
			}),
		}
	}
}

impl Add<u8> for Bytes {
	type Output = Self;

	fn add(self, rhs: u8) -> Self::Output {
		Add::add(self, Self::Single(rhs))
	}
}

impl Add<&u8> for Bytes {
	type Output = Self;

	fn add(self, rhs: &u8) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<Vec<u8>> for Bytes {
	type Output = Self;

	fn add(self, rhs: Vec<u8>) -> Self::Output {
		Add::add(self, Self::Many(rhs))
	}
}

impl AddAssign for Bytes {
	fn add_assign(&mut self, rhs: Self) {
		self.extend(rhs.into_vec());
	}
}

impl AddAssign<u8> for Bytes {
	fn add_assign(&mut self, rhs: u8) {
		self.extend(core::iter::once(rhs));
	}
}

impl AddAssign<&u8> for Bytes {
	fn add_assign(&mut self, rhs: &u8) {
		AddAssign::add_assign(self, *rhs);
	}
}

impl AddAssign<Vec<u8>> for Bytes {
	fn add_assign(&mut self, rhs: Vec<u8>) {
		self.extend(rhs);
	}
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Bytes {
	fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
		Ok(match (u64::from(u32::arbitrary(u)?) * 2u64) >> 32 {
			0 => Self::Single(arbitrary::Arbitrary::arbitrary(u)?),
			1 => Self::Many(arbitrary::Arbitrary::arbitrary(u)?),
			_ => unreachable!(),
		})
	}

	fn try_size_hint(
		depth: usize,
	) -> arbitrary::Result<(usize, Option<usize>), arbitrary::MaxRecursionReached> {
		Ok(arbitrary::size_hint::and(
			u32::try_size_hint(depth)?,
			arbitrary::size_hint::try_recursion_guard(depth, |depth| {
				Ok(arbitrary::size_hint::or_all(&[
					Ok(arbitrary::size_hint::and_all(&[u8::try_size_hint(depth)?]))?,
					Ok(arbitrary::size_hint::and_all(&[Vec::<u8>::try_size_hint(
						depth,
					)?]))?,
				]))
			})?,
		))
	}
}

impl Extend<u8> for Bytes {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = u8>,
	{
		self.as_mut_vec().extend(iter);

		self.shrink();
	}
}

impl From<u8> for Bytes {
	fn from(value: u8) -> Self {
		Self::Single(value)
	}
}

impl From<Vec<u8>> for Bytes {
	fn from(value: Vec<u8>) -> Self {
		Self::from_iter(value)
	}
}

impl<const N: usize> From<[u8; N]> for Bytes {
	fn from(value: [u8; N]) -> Self {
		Self::from_iter(value)
	}
}

impl FromIterator<u8> for Bytes {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = u8>,
	{
		let mut iter = iter.into_iter().peekable();
		let byte = iter.next();

		match byte {
			None => Self::Many(Vec::new()),
			Some(b) => {
				if iter.peek().is_none() {
					Self::Single(b)
				} else {
					let mut this = Self::Many(vec![b]);
					this.extend(iter);
					this
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use alloc::{vec, vec::Vec};

	use super::Bytes;

	#[test]
	fn add_works() {
		let left = Bytes::Single(1);
		let right = Bytes::Single(5);

		assert_eq!((left + right).into_vec(), [1, 5]);

		let left = Bytes::Many(vec![1, 2, 3, 4]);
		let right = Bytes::Single(5);

		assert_eq!((left + right).into_vec(), [1, 2, 3, 4, 5]);

		let left = Bytes::Single(1);
		let right = Bytes::Many(vec![2, 3, 4]);

		assert_eq!((left + right).into_vec(), [1, 2, 3, 4]);

		let left = Bytes::Many(vec![1, 2]);
		let right = Bytes::Many(vec![3, 4]);
		assert_eq!((left + right).into_vec(), [1, 2, 3, 4]);
	}

	#[test]
	fn add_assign_works() {
		let mut left = Bytes::Single(8);

		left += Bytes::Single(4);

		assert_eq!(left.into_vec(), [8, 4]);
	}

	#[test]
	fn from_iter_works() {
		let first = Bytes::from_iter([]);

		assert_eq!(first, Bytes::Many(Vec::new()));

		let only_one = Bytes::from_iter([5]);

		assert_eq!(only_one, Bytes::Single(5));

		let many = Bytes::from_iter([1, 2, 3, 4, 5]);

		assert_eq!(many, Bytes::Many(vec![1, 2, 3, 4, 5]));
	}
}
