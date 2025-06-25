use crate::Offset;

pub trait PtrMovement {
	fn ptr_movement(&self) -> Option<Offset>;

	fn might_move_ptr(&self) -> bool {
		!matches!(self.ptr_movement(), Some(Offset(0)))
	}
}

impl<T: PtrMovement> PtrMovement for [T] {
	#[inline]
	fn ptr_movement(&self) -> Option<Offset> {
		self.iter().try_fold(Offset(0), |a, b| {
			let ptr_movement = b.ptr_movement()?;

			Some(a + ptr_movement)
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::{Instruction, Offset, PtrMovement};

	#[test]
	fn no_movement() {
		assert_eq!(
			[Instruction::inc_val(2), Instruction::set_val(4)].ptr_movement(),
			Some(Offset(0))
		);
	}

	#[test]
	fn basic_movement() {
		assert_eq!([Instruction::move_ptr(7)].ptr_movement(), Some(Offset(7)));
		assert_eq!(
			[
				Instruction::move_ptr(7),
				Instruction::inc_val(21),
				Instruction::move_ptr(-7)
			]
			.ptr_movement(),
			Some(Offset(0))
		);
	}

	#[test]
	fn bad_movement() {
		assert_eq!(
			[
				Instruction::find_zero(3),
				Instruction::move_ptr(-3),
				Instruction::set_val(2)
			]
			.ptr_movement(),
			None
		);
	}
}
