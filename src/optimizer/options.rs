use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizerOptions {
	pub combine_instructions: bool,
	pub verbose: bool,
}

impl OptimizerOptions {
	#[must_use]
	pub const fn o0() -> Self {
		Self {
			combine_instructions: false,
			verbose: false,
		}
	}

	#[must_use]
	pub const fn o1() -> Self {
		Self {
			combine_instructions: true,
			verbose: false,
		}
	}

	#[must_use]
	pub const fn o2() -> Self {
		Self {
			combine_instructions: true,
			verbose: false,
		}
	}

	#[must_use]
	pub const fn o3() -> Self {
		Self {
			combine_instructions: true,
			verbose: false,
		}
	}

    #[must_use]
	pub const fn and_verbose(mut self, verbose: bool) -> Self {
		Self { verbose, ..self }
	}
}

impl Default for OptimizerOptions {
	fn default() -> Self {
		Self::o0()
	}
}
