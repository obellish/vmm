use core::num::NonZero;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Config {
	pub use_indices: bool,
	pub error_on_excess_data: bool,
	pub max_size: Option<NonZero<usize>>,
}

impl Config {
	#[must_use]
	pub const fn new(use_indices: bool, error_on_excess_data: bool, max_size: usize) -> Self {
		Self {
			use_indices,
			error_on_excess_data,
			max_size: NonZero::new(max_size),
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self::new(false, true, 0)
	}
}
