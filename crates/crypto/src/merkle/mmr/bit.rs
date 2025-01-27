#[repr(transparent)]
pub struct TrueBitPositionIterator {
	value: usize,
}

impl TrueBitPositionIterator {
	pub const fn new(value: usize) -> Self {
		Self { value }
	}
}

impl DoubleEndedIterator for TrueBitPositionIterator {
	fn next_back(&mut self) -> Option<Self::Item> {
		let zeros = self.value.leading_zeros();

		if zeros == usize::BITS {
			None
		} else {
			let bit_position = usize::BITS - zeros - 1;
			let mask = 1 << bit_position;
			self.value ^= mask;
			Some(bit_position)
		}
	}
}

impl Iterator for TrueBitPositionIterator {
	type Item = u32;

	fn next(&mut self) -> Option<Self::Item> {
		let zeros = self.value.trailing_zeros();

		if zeros == usize::BITS {
			None
		} else {
			let bit_position = zeros;
			let mask = 1 << bit_position;
			self.value ^= mask;
			Some(bit_position)
		}
	}
}
