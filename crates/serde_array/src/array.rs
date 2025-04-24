use core::ops::{Deref, DerefMut, Index, IndexMut};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::const_generics::{BigArray, PartiallyInitialized};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Array<T, const N: usize>(pub [T; N]);

impl<T: Default, const N: usize> Default for Array<T, N> {
	fn default() -> Self {
		let arr = {
			let mut arr = PartiallyInitialized::<T, N>::new();
			unsafe {
				{
					let p = arr.0.as_mut().unwrap();
					for i in 0..N {
						let p = p.as_mut_ptr().cast::<T>().wrapping_add(i);
						core::ptr::write(p, Default::default());
						arr.1 += 1;
					}
				}

				arr.0.take().unwrap().assume_init()
			}
		};

		Self(arr)
	}
}

impl<T, const N: usize> Deref for Array<T, N> {
	type Target = [T; N];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T, const N: usize> DerefMut for Array<T, N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'de, T, const N: usize> Deserialize<'de> for Array<T, N>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Ok(Self(<[T; N] as BigArray<'de, T>>::deserialize(
			deserializer,
		)?))
	}
}

impl<T, const N: usize> From<[T; N]> for Array<T, N> {
	fn from(value: [T; N]) -> Self {
		Self(value)
	}
}

impl<T, const N: usize> From<Array<T, N>> for [T; N] {
	fn from(value: Array<T, N>) -> Self {
		value.0
	}
}

impl<T, I, const N: usize> Index<I> for Array<T, N>
where
	[T]: Index<I>,
{
	type Output = <[T] as Index<I>>::Output;

	fn index(&self, index: I) -> &Self::Output {
		Index::index(&self.0 as &[T], index)
	}
}

impl<T, I, const N: usize> IndexMut<I> for Array<T, N>
where
	[T]: IndexMut<I>,
{
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		IndexMut::index_mut(&mut self.0 as &mut [T], index)
	}
}

impl<T: Serialize, const N: usize> Serialize for Array<T, N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		<[T; N] as BigArray<T>>::serialize(&self.0, serializer)
	}
}
