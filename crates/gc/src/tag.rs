#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tag {
	None,
	First,
	Second,
	Both,
}

impl Tag {
	pub(super) const fn value(self) -> usize {
		match self {
			Self::None => 0,
			Self::First => 1,
			Self::Second => 2,
			Self::Both => 3,
		}
	}

	pub(super) fn into_tag<P>(ptr: *const P) -> Self {
		match (matches!(ptr as usize & 1, 1), matches!(ptr as usize & 2, 2)) {
			(false, false) => Self::None,
			(true, false) => Self::First,
			(false, true) => Self::Second,
			(true, true) => Self::Both,
		}
	}

    pub(super) fn update_tag<P>(ptr: *const P, tag: Self) -> *const P {
        (((ptr as usize) & (!3)) | tag.value()) as *const P
    }

    pub(super) fn unset_tag<P>(ptr: *const P) -> *const P {
        ((ptr as usize) & (!3)) as *const P
    }
}
