use std::fmt::{Display, Formatter, Result as FmtResult};

use bevy_reflect::prelude::*;

use super::{Datum, InternalData};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum Mutator {
	Set(String, Datum),
	Increment(String, Datum),
	Decrement(String, Datum),
}

impl Mutator {
	pub fn apply(&self, data: &mut InternalData) {
		match self {
			Self::Set(key, value) => {
				data.insert(key.to_string(), *value);
			}
			Self::Increment(key, value) => {
				if let Some(current) = data.get_mut(key) {
					*current += *value;
				}
			}
			Self::Decrement(key, value) => {
				if let Some(current) = data.get_mut(key) {
					*current -= *value;
				}
			}
		}
	}
}

impl Display for Mutator {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Set(k, v) => {
				f.write_str(k)?;
				f.write_str(" = ")?;
				Display::fmt(&v, f)
			}
			Self::Increment(k, v) => {
				f.write_str(k)?;
				f.write_str(" + ")?;
				Display::fmt(&v, f)
			}
			Self::Decrement(k, v) => {
				f.write_str(k)?;
				f.write_str(" - ")?;
				Display::fmt(&v, f)
			}
		}
	}
}
