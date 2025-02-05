#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProcedureIndex(u16);

impl ProcedureIndex {
	#[must_use]
	pub fn new(id: usize) -> Self {
		Self::new_unchecked(
			id.try_into()
				.expect("invalid procedure index: too many procedures"),
		)
	}

	#[must_use]
	pub const fn new_unchecked(id: u16) -> Self {
		Self(id)
	}

	#[must_use]
	pub const fn as_usize(self) -> usize {
		self.0 as usize
	}
}
