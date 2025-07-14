use core::{
	num::NonZeroU32,
	sync::atomic::{AtomicU32, Ordering},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct BasicBlockId(pub(crate) NonZeroU32);

impl BasicBlockId {
	const BASIC_BLOCK_ID_START: Self = unsafe { Self(NonZeroU32::new_unchecked(1)) };

	pub(crate) fn next() -> Self {
		static NEXT_VALUE: AtomicU32 = AtomicU32::new(BasicBlockId::BASIC_BLOCK_ID_START.0.get());
		let id = NEXT_VALUE.fetch_add(1, Ordering::Relaxed);
		unsafe { Self(NonZeroU32::new_unchecked(id)) }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ValueId(pub(crate) NonZeroU32);

impl ValueId {
	const GLOBAL_VALUE_ID_START: Self = unsafe { Self(NonZeroU32::new_unchecked(1)) };
	const LOCAL_VALUE_ID_START: Self = unsafe { Self(NonZeroU32::new_unchecked(0x4000_0000)) };

	pub(crate) fn global() -> Self {
		static NEXT_GLOBAL_VALUE: AtomicU32 =
			AtomicU32::new(ValueId::GLOBAL_VALUE_ID_START.0.get());
		let id = NEXT_GLOBAL_VALUE.fetch_add(1, Ordering::Relaxed);
		unsafe { Self(NonZeroU32::new_unchecked(id)) }
	}

	pub(crate) fn local() -> Self {
		static NEXT_LOCAL_VALUE: AtomicU32 = AtomicU32::new(ValueId::LOCAL_VALUE_ID_START.0.get());
		let id = NEXT_LOCAL_VALUE.fetch_add(1, Ordering::Relaxed);
		unsafe { Self(NonZeroU32::new_unchecked(id)) }
	}

	#[must_use]
	pub fn is_global(self) -> bool {
		self.0 >= Self::GLOBAL_VALUE_ID_START.0 && self.0 < Self::LOCAL_VALUE_ID_START.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FunctionId(pub(crate) NonZeroU32);

impl FunctionId {
	const NEXT_VALUE: Self = unsafe { Self(NonZeroU32::new_unchecked(1)) };

	pub(crate) fn next() -> Self {
		static NEXT_VALUE: AtomicU32 = AtomicU32::new(FunctionId::NEXT_VALUE.0.get());
		let id = NEXT_VALUE.fetch_add(1, Ordering::Relaxed);
		unsafe { Self(NonZeroU32::new_unchecked(id)) }
	}
}
