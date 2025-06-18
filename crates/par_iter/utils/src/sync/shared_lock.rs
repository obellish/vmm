use std::{
	boxed::Box,
	cell::UnsafeCell,
	collections::HashMap,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	marker::PhantomData,
	mem,
	ops::{Deref, DerefMut},
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

	pub fn try_read(&self) -> TryLockResult<ShardedLockReadGuard<'_, T>> {
		let current_index = current_index().unwrap_or(0);
		let shard_index = current_index & (self.shards.len() - 1);

		match self.shards[shard_index].lock.try_read() {
			Ok(guard) => Ok(ShardedLockReadGuard {
				lock: self,
				_guard: guard,
				marker: PhantomData,
			}),
			Err(TryLockError::Poisoned(err)) => {
				let guard = ShardedLockReadGuard {
					lock: self,
					_guard: err.into_inner(),
					marker: PhantomData,
				};

				Err(TryLockError::Poisoned(PoisonError::new(guard)))
			}
			Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
		}
	}

	pub fn read(&self) -> LockResult<ShardedLockReadGuard<'_, T>> {
		let current_index = current_index().unwrap_or(0);
		let shard_index = current_index & (self.shards.len() - 1);

		match self.shards[shard_index].lock.read() {
			Ok(guard) => Ok(ShardedLockReadGuard {
				lock: self,
				_guard: guard,
				marker: PhantomData,
			}),
			Err(err) => Err(PoisonError::new(ShardedLockReadGuard {
				lock: self,
				_guard: err.into_inner(),
				marker: PhantomData,
			})),
		}
	}

	pub fn try_write(&self) -> TryLockResult<ShardedLockWriteGuard<'_, T>> {
		let mut poisoned = false;
		let mut blocked = None;

		for (i, shard) in self.shards.iter().enumerate() {
			let guard = match shard.lock.try_write() {
				Ok(guard) => guard,
				Err(TryLockError::Poisoned(err)) => {
					poisoned = true;
					err.into_inner()
				}
				Err(TryLockError::WouldBlock) => {
					blocked = Some(i);
					break;
				}
			};

			unsafe {
				let guard: RwLockWriteGuard<'static, ()> = mem::transmute(guard);
				let dest: *mut _ = shard.write_guard.get();
				*dest = Some(guard);
			}
		}

		if let Some(i) = blocked {
			for shard in self.shards[0..i].iter().rev() {
				unsafe {
					let dest: *mut _ = shard.write_guard.get();
					let guard = (*dest).take();
					drop(guard);
				}
			}

			Err(TryLockError::WouldBlock)
		} else if poisoned {
			let guard = ShardedLockWriteGuard {
				lock: self,
				marker: PhantomData,
			};
			Err(TryLockError::Poisoned(PoisonError::new(guard)))
		} else {
			Ok(ShardedLockWriteGuard {
				lock: self,
				marker: PhantomData,
			})
		}
	}

	pub fn write(&self) -> LockResult<ShardedLockWriteGuard<'_, T>> {
		let mut poisoned = false;

		for shard in &self.shards {
			let guard = match shard.lock.write() {
				Ok(guard) => guard,
				Err(err) => {
					poisoned = true;
					err.into_inner()
				}
			};

			unsafe {
				let guard: RwLockWriteGuard<'_, ()> = guard;
				let guard: RwLockWriteGuard<'static, ()> = mem::transmute(guard);
				let dest: *mut _ = shard.write_guard.get();
				*dest = Some(guard);
			}
		}

		if poisoned {
			Err(PoisonError::new(ShardedLockWriteGuard {
				lock: self,
				marker: PhantomData,
			}))
		} else {
			Ok(ShardedLockWriteGuard {
				lock: self,
				marker: PhantomData,
			})
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

impl<T> Debug for ShardedLock<T>
where
	T: ?Sized + Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self.try_read() {
			Ok(guard) => f
				.debug_struct("ShardedLock")
				.field("data", &&*guard)
				.finish(),
			Err(TryLockError::Poisoned(err)) => f
				.debug_struct("ShardedLock")
				.field("data", &&**err.get_ref())
				.finish(),
			Err(TryLockError::WouldBlock) => {
				struct LockedPlaceholder;
				impl Debug for LockedPlaceholder {
					fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
						f.write_str("<locked>")
					}
				}

				f.debug_struct("ShardedLock")
					.field("data", &LockedPlaceholder)
					.finish()
			}
		}
	}
}

impl<T: Default> Default for ShardedLock<T> {
	fn default() -> Self {
		Self::new(Default::default())
	}
}

impl<T> From<T> for ShardedLock<T> {
	fn from(value: T) -> Self {
		Self::new(value)
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

impl<T: Debug> Debug for ShardedLockReadGuard<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("ShardedLockReadGuard")
			.field("lock", &self.lock)
			.finish()
	}
}

impl<T: ?Sized> Deref for ShardedLockReadGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.lock.value.get() }
	}
}

impl<T> Display for ShardedLockReadGuard<'_, T>
where
	T: ?Sized + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&**self, f)
	}
}

unsafe impl<T> Sync for ShardedLockReadGuard<'_, T> where T: ?Sized + Sync {}

#[clippy::has_significant_drop]
pub struct ShardedLockWriteGuard<'a, T: ?Sized> {
	lock: &'a ShardedLock<T>,
	marker: PhantomData<RwLockWriteGuard<'a, T>>,
}

impl<T> Debug for ShardedLockWriteGuard<'_, T>
where
	T: ?Sized + Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("ShardedLockWriteGuard")
			.field("lock", &self.lock)
			.finish()
	}
}

impl<T: ?Sized> Deref for ShardedLockWriteGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.lock.value.get() }
	}
}

impl<T: ?Sized> DerefMut for ShardedLockWriteGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.lock.value.get() }
	}
}

impl<T> Display for ShardedLockWriteGuard<'_, T>
where
	T: ?Sized + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&**self, f)
	}
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

struct Registration {
	index: usize,
	thread_id: ThreadId,
}

impl Drop for Registration {
	fn drop(&mut self) {
		let mut indices = thread_indices().lock().unwrap();
		indices.mapping.remove(&self.thread_id);
		indices.free_list.push(self.index);
	}
}

struct ThreadIndices {
	mapping: HashMap<ThreadId, usize>,
	free_list: Vec<usize>,
	next_index: usize,
}

fn thread_indices() -> &'static Mutex<ThreadIndices> {
	static THREAD_INDICES: OnceLock<Mutex<ThreadIndices>> = OnceLock::new();

	THREAD_INDICES.get_or_init(|| {
		Mutex::new(ThreadIndices {
			mapping: HashMap::new(),
			free_list: Vec::new(),
			next_index: 0,
		})
	})
}

fn current_index() -> Option<usize> {
	REGISTRATION.try_with(|reg| reg.index).ok()
}

std::thread_local! {
	static REGISTRATION: Registration = {
		let thread_id = thread::current().id();
		let mut indices = thread_indices().lock().unwrap();

		let index = if let Some(i) = indices.free_list.pop() { i } else {
				  let i = indices.next_index;
				  indices.next_index += 1;
				  i
			  };
		indices.mapping.insert(thread_id, index);

		Registration { index, thread_id }
	}
}
