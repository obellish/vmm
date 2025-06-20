mod boxed;
mod ptr;
mod vec;

pub use self::{boxed::*, ptr::*, vec::*};

#[cfg(test)]
mod tests {
	use crate::{BoxTape, PtrTape, TAPE_SIZE, Tape, VecTape};

	fn check_length<T: Tape>() {
		let mut tape = T::default();

		tape.init();

		assert_eq!(tape.as_slice().len(), TAPE_SIZE);
	}

	#[test]
	fn is_box_correct_length() {
		check_length::<BoxTape>();
	}

	#[test]
	fn is_ptr_correct_length() {
		check_length::<PtrTape>();
	}

	#[test]
	fn is_vec_correct_length() {
		check_length::<VecTape>();
	}
}
