use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::debuginfo::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblyOp {
	location: Option<Location>,
	context_name: String,
	op: String,
	num_cycles: u8,
	should_break: bool,
}

impl AssemblyOp {
	#[must_use]
	pub const fn new(
		location: Option<Location>,
		context_name: String,
		op: String,
		num_cycles: u8,
		should_break: bool,
	) -> Self {
		Self {
			location,
			context_name,
			op,
			num_cycles,
			should_break,
		}
	}

	#[must_use]
	pub const fn location(&self) -> Option<&Location> {
		self.location.as_ref()
	}

	pub fn set_location(&mut self, location: Location) {
		self.location = Some(location);
	}

	#[must_use]
	pub fn context_name(&self) -> &str {
		&self.context_name
	}

	#[must_use]
	pub const fn num_cycles(&self) -> u8 {
		self.num_cycles
	}

	pub fn num_cycles_mut(&mut self) -> &mut u8 {
		&mut self.num_cycles
	}

	pub fn set_num_cycles(&mut self, num_cycles: u8) {
		*self.num_cycles_mut() = num_cycles;
	}

	#[must_use]
	pub fn op(&self) -> &str {
		&self.op
	}

	#[must_use]
	pub const fn should_break(&self) -> bool {
		self.should_break
	}
}

impl Display for AssemblyOp {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("context=")?;
		f.write_str(self.context_name())?;
		f.write_str(", operation=")?;
		f.write_str(self.op())?;
		f.write_str(", cost=")?;
		Display::fmt(&self.num_cycles(), f)
	}
}
