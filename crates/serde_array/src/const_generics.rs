use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
	mem::MaybeUninit,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, SeqAccess, Visitor},
	ser::SerializeTuple,
};

pub struct PartiallyInitialized<T, const N: usize>(pub Option<MaybeUninit<[T; N]>>, pub usize);

impl<T, const N: usize> PartiallyInitialized<T, N> {
	pub const fn new() -> Self {
		Self(Some(MaybeUninit::uninit()), 0)
	}
}

impl<T, const N: usize> Drop for PartiallyInitialized<T, N> {
	fn drop(&mut self) {
		if !core::mem::needs_drop::<T>() {
			return;
		}

		if let Some(array) = &mut self.0 {
			while self.1 > 0 {
				self.1 -= 1;
				let offs = self.1;
				let p = array.as_mut_ptr().cast::<T>().wrapping_add(offs);
				unsafe {
					core::ptr::drop_in_place(p);
				}
			}
		}
	}
}

struct ArrayVisitor<A: ?Sized> {
	elements: PhantomData<A>,
}

impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<[T; N]>
where
	T: Deserialize<'de>,
{
	type Value = [T; N];

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		write!(formatter, "an array of length {N}")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		unsafe {
			let mut arr = PartiallyInitialized::<T, N>::new();

			{
				let p = arr.0.as_mut().unwrap();
				for i in 0..N {
					let p = p.as_mut_ptr().cast::<T>().wrapping_add(i);
					let val = seq
						.next_element()?
						.ok_or_else(|| DeError::invalid_length(i, &self))?;

					core::ptr::write(p, val);
					arr.1 += 1;
				}
			}

			let initialized = arr.0.take().unwrap().assume_init();
			Ok(initialized)
		}
	}
}

pub trait BigArray<'de, T>: Sized {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		T: Serialize;

	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
		T: Deserialize<'de>;
}

impl<'de, T, const N: usize> BigArray<'de, T> for [T; N] {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		T: Serialize,
	{
		let mut seq = serializer.serialize_tuple(self.len())?;
		for elem in &self[..] {
			seq.serialize_element(elem)?;
		}

		seq.end()
	}

	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
		T: Deserialize<'de>,
	{
		let visitor = ArrayVisitor {
			elements: PhantomData,
		};

		deserializer.deserialize_tuple(N, visitor)
	}
}
