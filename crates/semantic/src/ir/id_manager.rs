use std::{
	num::NonZero,
	sync::atomic::{AtomicU32, Ordering},
};

pub(crate) type ValueId = NonZero<u32>;

const GLOBAL_VALUE_ID_START: ValueId = unsafe { NonZero::new_unchecked(1) };

const LOCAL_VALUE_ID_START: ValueId = unsafe { NonZero::new_unchecked(0x4000_0000) };

pub(crate) type BasicBlockId = NonZero<u32>;

const BASIC_BLOCK_ID_START: BasicBlockId = unsafe { NonZero::new_unchecked(1) };

pub(crate) type FunctionId = NonZero<u32>;

const FUNC_ID_START: FunctionId = unsafe { NonZero::new_unchecked(1) };

pub(crate) fn next_global_value_id() -> ValueId {
	static NEXT_GLOBAL_VALUE_ID: AtomicU32 = AtomicU32::new(GLOBAL_VALUE_ID_START.get());
	let id = NEXT_GLOBAL_VALUE_ID.fetch_add(1, Ordering::Relaxed);
	unsafe { NonZero::new_unchecked(id) }
}

pub(crate) fn next_local_value_id() -> ValueId {
	static NEXT_LOCAL_VALUE_ID: AtomicU32 = AtomicU32::new(LOCAL_VALUE_ID_START.get());
	let id = NEXT_LOCAL_VALUE_ID.fetch_add(1, Ordering::Relaxed);
	unsafe { NonZero::new_unchecked(id) }
}

pub(crate) fn is_global_id(value: ValueId) -> bool {
	value >= GLOBAL_VALUE_ID_START && value < LOCAL_VALUE_ID_START
}

pub(crate) fn next_basic_block_id() -> BasicBlockId {
	static NEXT_BASIC_BLOCK_ID: AtomicU32 = AtomicU32::new(BASIC_BLOCK_ID_START.get());
	let id = NEXT_BASIC_BLOCK_ID.fetch_add(1, Ordering::Relaxed);
	unsafe { NonZero::new_unchecked(id) }
}

pub(crate) fn next_func_id() -> FunctionId {
	static NEXT_FUNC_ID: AtomicU32 = AtomicU32::new(FUNC_ID_START.get());
	let id = NEXT_FUNC_ID.fetch_add(1, Ordering::Relaxed);
	unsafe { NonZero::new_unchecked(id) }
}
