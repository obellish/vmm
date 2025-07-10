#![allow(clippy::match_wildcard_for_single_variants)]

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use tap::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Float {
	F32(f32),
	F64(f64),
}

impl Debug for Float {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for Float {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::F32(float) => Display::fmt(&float, f),
			Self::F64(float) => Display::fmt(&float, f),
		}
	}
}

impl From<f32> for Float {
	fn from(value: f32) -> Self {
		Self::F32(value)
	}
}

impl From<f64> for Float {
	fn from(value: f64) -> Self {
		Self::F64(value)
	}
}

impl PartialEq<f32> for Float {
	fn eq(&self, other: &f32) -> bool {
		match self {
			Self::F32(f) => PartialEq::eq(f, other),
			_ => false,
		}
	}
}

impl PartialEq<f64> for Float {
	fn eq(&self, other: &f64) -> bool {
		match self {
			Self::F32(f) => PartialEq::eq(&(*f).convert::<f64>(), other),
			Self::F64(f) => PartialEq::eq(f, other),
		}
	}
}
