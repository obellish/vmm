#[cfg(feature = "serde")]
mod serde;

use std::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	mem::{self, size_of},
};

#[repr(transparent)]
pub struct FamStructWrapper<T>
where
	T: Default + FamStruct,
{
	mem_allocator: Vec<T>,
}

impl<T> FamStructWrapper<T>
where
	T: Default + FamStruct,
{
	fn mem_allocator_len(fam_len: usize) -> Option<usize> {
		let wrapper_size_in_bytes =
			size_of::<T>().checked_add(fam_len.checked_mul(size_of::<T::Entry>())?)?;

		wrapper_size_in_bytes
			.checked_add(size_of::<T>().checked_sub(1)?)?
			.checked_div(size_of::<T>())
	}

	const fn fam_len(mem_allocator_len: usize) -> usize {
		if matches!(mem_allocator_len, 0) {
			return 0;
		}

		let array_size_in_bytes = (mem_allocator_len - 1) * size_of::<T>();
		array_size_in_bytes / size_of::<T::Entry>()
	}

	pub fn new(num_elements: usize) -> Result<Self, Error> {
		if num_elements > T::max_len() {
			return Err(Error::SizeLimitExceeded);
		}

		let required_mem_allocator_capacity =
			Self::mem_allocator_len(num_elements).ok_or(Error::SizeLimitExceeded)?;

		let mut mem_allocator = Vec::with_capacity(required_mem_allocator_capacity);
		mem_allocator.push(T::default());
		for _ in 1..required_mem_allocator_capacity {
			mem_allocator.push(unsafe { mem::zeroed() });
		}

		unsafe {
			mem_allocator[0].set_len(num_elements);
		}

		Ok(Self { mem_allocator })
	}

	pub fn from_entries(entries: &[T::Entry]) -> Result<Self, Error> {
		let mut adapter = Self::new(entries.len())?;

		{
			let wrapper_entries = unsafe { adapter.as_mut_fam_struct().as_mut_slice() };
			wrapper_entries.copy_from_slice(entries);
		}

		Ok(adapter)
	}

	#[must_use]
	pub const unsafe fn from_raw(content: Vec<T>) -> Self {
		Self {
			mem_allocator: content,
		}
	}

	#[must_use]
	pub fn into_raw(self) -> Vec<T> {
		self.mem_allocator
	}

	#[must_use]
	pub fn as_fam_struct(&self) -> &T {
		&self.mem_allocator[0]
	}

	pub unsafe fn as_mut_fam_struct(&mut self) -> &mut T {
		&mut self.mem_allocator[0]
	}

	#[must_use]
	pub fn as_fam_struct_ptr(&self) -> *const T {
		self.as_fam_struct()
	}

	pub fn as_mut_fam_struct_ptr(&mut self) -> *mut T {
		unsafe { self.as_mut_fam_struct() }
	}

	#[must_use]
	pub fn as_slice(&self) -> &[T::Entry] {
		self.as_fam_struct().as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [T::Entry] {
		unsafe { self.as_mut_fam_struct() }.as_mut_slice()
	}

	fn len(&self) -> usize {
		self.as_fam_struct().len()
	}

	fn capacity(&self) -> usize {
		Self::fam_len(self.mem_allocator.capacity())
	}

	fn reserve(&mut self, additional: usize) -> Result<(), Error> {
		let desired_capacity = self.len() + additional;
		if desired_capacity <= self.capacity() {
			return Ok(());
		}

		let current_mem_allocator_len = self.mem_allocator.len();
		let required_mem_allocator_len =
			Self::mem_allocator_len(desired_capacity).ok_or(Error::SizeLimitExceeded)?;
		let additional_mem_allocator_len = required_mem_allocator_len - current_mem_allocator_len;

		self.mem_allocator.reserve(additional_mem_allocator_len);

		Ok(())
	}

	fn set_len(&mut self, len: usize) -> Result<(), Error> {
		let additional_elements = isize::try_from(len)
			.and_then(|len| isize::try_from(self.len()).map(|self_len| len - self_len))
			.map_err(|_| Error::SizeLimitExceeded)?;

		if matches!(additional_elements, 0) {
			return Ok(());
		}

		if additional_elements > 0 {
			if len > T::max_len() {
				return Err(Error::SizeLimitExceeded);
			}

			self.reserve(additional_elements as usize)?;
		}

		let current_mem_allocator_len = self.mem_allocator.len();
		let required_mem_allocator_len =
			Self::mem_allocator_len(len).ok_or(Error::SizeLimitExceeded)?;

		unsafe {
			self.mem_allocator.set_len(required_mem_allocator_len);
		}

		for i in current_mem_allocator_len..required_mem_allocator_len {
			self.mem_allocator[i] = unsafe { mem::zeroed() }
		}

		unsafe {
			self.as_mut_fam_struct().set_len(len);
		}

		if additional_elements < 0 {
			self.mem_allocator.shrink_to_fit();
		}

		Ok(())
	}

	pub fn push(&mut self, entry: T::Entry) -> Result<(), Error> {
		let new_len = self.len() + 1;
		self.set_len(new_len)?;
		self.as_mut_slice()[new_len - 1] = entry;

		Ok(())
	}

	pub fn retain<F>(&mut self, mut f: F)
	where
		F: FnMut(&T::Entry) -> bool,
	{
		let mut num_kept_entries = 0;
		{
			let entries = self.as_mut_slice();
			for entry_idx in 0..entries.len() {
				let keep = f(&entries[entry_idx]);
				if keep {
					entries[num_kept_entries] = entries[entry_idx];
					num_kept_entries += 1;
				}
			}
		}

		self.set_len(num_kept_entries).expect("invalid length");
	}
}

impl<T> Clone for FamStructWrapper<T>
where
	T: Default + FamStruct,
{
	fn clone(&self) -> Self {
		let required_mem_allocator_capacity =
			Self::mem_allocator_len(self.as_slice().len()).unwrap();

		let mut mem_allocator = Vec::with_capacity(required_mem_allocator_capacity);

		unsafe {
			let fam_struct: T = std::ptr::read(self.as_fam_struct_ptr());
			mem_allocator.push(fam_struct);
		}

		for _ in 1..required_mem_allocator_capacity {
			mem_allocator.push(unsafe { mem::zeroed() });
		}

		let mut adapter = Self { mem_allocator };

		{
			let wrapper_entries = adapter.as_mut_slice();
			wrapper_entries.copy_from_slice(self.as_slice());
		}

		adapter
	}
}

impl<T> Debug for FamStructWrapper<T>
where
	T: Debug + Default + FamStruct,
	T::Entry: Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("FamStructWrapper")
			.field("fam_struct", &self.as_fam_struct())
			.field("entries", &self.as_fam_struct().as_slice())
			.finish()
	}
}

impl<T> From<Vec<T>> for FamStructWrapper<T>
where
	T: Default + FamStruct,
{
	fn from(value: Vec<T>) -> Self {
		unsafe { Self::from_raw(value) }
	}
}

impl<T> PartialEq for FamStructWrapper<T>
where
	T: Default + FamStruct + PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.as_fam_struct() == other.as_fam_struct() && self.as_slice() == other.as_slice()
	}
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
	SizeLimitExceeded,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::SizeLimitExceeded => f.write_str("the max size has been exceeded"),
		}
	}
}

impl std::error::Error for Error {}

#[allow(clippy::len_without_is_empty)]
pub unsafe trait FamStruct {
	type Entry: Copy + PartialEq;

	fn len(&self) -> usize;

	unsafe fn set_len(&mut self, len: usize);

	fn max_len() -> usize;

	fn as_slice(&self) -> &[Self::Entry];

	fn as_mut_slice(&mut self) -> &mut [Self::Entry];
}
