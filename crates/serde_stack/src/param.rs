#[derive(Clone, Copy)]
pub struct Param {
	pub red_zone: usize,
	pub stack_size: usize,
}

impl Param {
	pub const fn new(red_zone: usize, stack_size: usize) -> Self {
		Self {
			red_zone,
			stack_size,
		}
	}
}

impl Default for Param {
	fn default() -> Self {
		Self::new(64 * 1024, 2 * 1024 * 1024)
	}
}
