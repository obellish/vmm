use std::{
	boxed::Box,
	cell::UnsafeCell,
	collections::HashMap,
	fmt,
	marker::PhantomData,
	mem,
	ops::{Deref, DerefMut},
	panic::{RefUnwindSafe, UnwindSafe},
	sync::{
		LockResult, Mutex, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError,
		TryLockResult,
	},
	thread::{self, ThreadId},
	vec::Vec,
};

use crate::{CachePadded, sync::once_lock::OnceLock};

const NUM_SHARDS: usize = 8;

pub struct ShardedLock<T: ?Sized> {
	shards: Box<[CachePadded<Shard>]>,
	value: UnsafeCell<T>,
}

impl<T: ?Sized> ShardedLock<T> {
	pub fn is_poisoned(&self) -> bool {
		self.shards[0].lock.is_poisoned()
	}

	pub fn get_mut(&mut self) -> LockResult<&mut T> {
		let is_poisoned = self.is_poisoned();
		let inner = unsafe { &mut *self.value.get() };
		if is_poisoned {
			Err(PoisonError::new(inner))
		} else {
			Ok(inner)
		}
	}
}

impl<T> ShardedLock<T> {
	pub fn new(value: T) -> Self {
		Self {
			shards: (0..NUM_SHARDS)
				.map(|_| {
					CachePadded::new(Shard {
						lock: RwLock::new(()),
						write_guard: UnsafeCell::new(None),
					})
				})
				.collect(),
			value: UnsafeCell::new(value),
		}
	}

	pub fn into_inner(self) -> LockResult<T> {
		let is_poisoned = self.is_poisoned();
		let inner = self.value.into_inner();

		if is_poisoned {
			Err(PoisonError::new(inner))
		} else {
			Ok(inner)
		}
	}
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for ShardedLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for ShardedLock<T> where T: ?Sized + Send + Sync {}

#[clippy::has_significant_drop]
pub struct ShardedLockReadGuard<'a, T: ?Sized> {
	lock: &'a ShardedLock<T>,
	_guard: RwLockReadGuard<'a, ()>,
	marker: PhantomData<RwLockReadGuard<'a, T>>,
}

impl<T: ?Sized> Deref for ShardedLockReadGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.lock.value.get() }
	}
}

unsafe impl<T> Sync for ShardedLockReadGuard<'_, T> where T: ?Sized + Sync {}

#[clippy::has_significant_drop]
pub struct ShardedLockWriteGuard<'a, T: ?Sized> {
	lock: &'a ShardedLock<T>,
	marker: PhantomData<RwLockWriteGuard<'a, T>>,
}

impl<T: ?Sized> Drop for ShardedLockWriteGuard<'_, T> {
	fn drop(&mut self) {
		for shard in self.lock.shards.iter().rev() {
			unsafe {
				let dest: *mut _ = shard.write_guard.get();
				let guard = (*dest).take();
				drop(guard);
			}
		}
	}
}

unsafe impl<T> Sync for ShardedLockWriteGuard<'_, T> where T: ?Sized + Sync {}

struct Shard {
	lock: RwLock<()>,
	write_guard: UnsafeCell<Option<RwLockWriteGuard<'static, ()>>>,
}
