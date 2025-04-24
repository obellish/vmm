use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scanner {
	source: String,
	current: usize,
	line: usize,
}

impl Scanner {
    #[must_use]
	pub const fn new(source: String) -> Self {
		Self {
			source,
			current: 0,
			line: 0,
		}
	}
}
