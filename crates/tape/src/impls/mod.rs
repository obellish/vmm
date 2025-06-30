mod boxed;
mod ptr;
mod stack;
mod vec;

pub use self::{boxed::*, ptr::*, stack::*, vec::*};

#[cfg(test)]
mod tests {
	use crate::{BoxTape, PtrTape, StackTape, TAPE_SIZE, Tape, VecTape};

	fn check_impl<T: Tape>() {
		let mut tape = T::default();

		tape.init();

		assert_eq!(tape.as_slice().len(), TAPE_SIZE);

		for (i, cell) in tape.as_slice().iter().copied().enumerate() {
			assert_eq!(cell.index(), Some(i));
		}
	}

	#[test]
	fn is_box_correct_impl() {
		check_impl::<BoxTape>();
	}

	#[test]
	fn is_ptr_correct_impl() {
		check_impl::<PtrTape>();
	}

	#[test]
	fn is_vec_correct_impl() {
		check_impl::<VecTape>();
	}

	#[test]
	fn is_stack_correct_impl() {
		check_impl::<StackTape>();
	}
}
