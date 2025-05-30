pub trait PtrMovement {
	fn ptr_movement(&self) -> Option<isize>;

	fn might_move_ptr(&self) -> bool {
		!matches!(self.ptr_movement(), Some(0))
	}
}

impl<T: PtrMovement> PtrMovement for [T] {
	fn ptr_movement(&self) -> Option<isize> {
		let mut sum = 0;

		for i in self {
			sum += i.ptr_movement()?;
		}

		Some(sum)
	}
}
