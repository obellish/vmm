use std::{mem::ManuallyDrop, slice};

#[must_use]
pub fn u8_vec_into_i8_vec(vec: Vec<u8>) -> Vec<i8> {
	let mut vec = ManuallyDrop::new(vec);
	unsafe { Vec::from_raw_parts(vec.as_mut_ptr().cast::<i8>(), vec.len(), vec.capacity()) }
}

#[must_use]
pub fn i8_vec_into_u8_vec(vec: Vec<i8>) -> Vec<u8> {
	let mut vec = ManuallyDrop::new(vec);
	unsafe { Vec::from_raw_parts(vec.as_mut_ptr().cast(), vec.len(), vec.capacity()) }
}

#[must_use]
pub const fn u8_slice_as_i8_slice(slice: &[u8]) -> &[i8] {
	unsafe { slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
}

#[must_use]
pub const fn i8_slice_as_u8_slice(slice: &[i8]) -> &[u8] {
	unsafe { slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
}
