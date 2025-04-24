mod scanner;
mod token;

use serde::{Deserialize, Serialize};

pub use self::{scanner::*, token::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compiler {
	scanner: Scanner,
}

impl Compiler {
	#[must_use]
	pub const fn new(source: String) -> Self {
		Self {
			scanner: Scanner::new(source),
		}
	}
}
