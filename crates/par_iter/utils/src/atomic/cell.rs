#![allow(clippy::unit_arg, clippy::ptr_as_ptr, clippy::ptr_cast_constness)]

use core::{
	cell::UnsafeCell,
	cmp,
	fmt::{Debug, Formatter, Result as FmtResult},
	mem::{self, ManuallyDrop, MaybeUninit},
	panic::{RefUnwindSafe, UnwindSafe},
	ptr,
	sync::atomic::{self, Ordering},
};

use super::seq_lock::SeqLock;
use crate::CachePadded;

macro_rules! atomic {
    // If values of type `$t` can be transmuted into values of the primitive atomic type `$atomic`,
    // declares variable `$a` of type `$atomic` and executes `$atomic_op`, breaking out of the loop.
    (@check, $t:ty, $atomic:ty, $a:ident, $atomic_op:expr) => {
        if can_transmute::<$t, $atomic>() {
            let $a: &$atomic;
            break $atomic_op;
        }
    };

    // If values of type `$t` can be transmuted into values of a primitive atomic type, declares
    // variable `$a` of that type and executes `$atomic_op`. Otherwise, just executes
    // `$fallback_op`.
    ($t:ty, $a:ident, $atomic_op:expr, $fallback_op:expr) => {
        loop {
            atomic!(@check, $t, AtomicUnit, $a, $atomic_op);

            // Always use fallback for now on environments that do not support inline assembly.
            #[cfg(not(miri))]
            atomic_maybe_uninit::cfg_has_atomic_cas! {
                atomic_maybe_uninit::cfg_has_atomic_8! {
                    atomic!(@check, $t, atomic_maybe_uninit::AtomicMaybeUninit<u8>, $a, $atomic_op);
                }
                atomic_maybe_uninit::cfg_has_atomic_16! {
                    atomic!(@check, $t, atomic_maybe_uninit::AtomicMaybeUninit<u16>, $a, $atomic_op);
                }
                atomic_maybe_uninit::cfg_has_atomic_32! {
                    atomic!(@check, $t, atomic_maybe_uninit::AtomicMaybeUninit<u32>, $a, $atomic_op);
                }
                atomic_maybe_uninit::cfg_has_atomic_64! {
                    atomic!(@check, $t, atomic_maybe_uninit::AtomicMaybeUninit<u64>, $a, $atomic_op);
                }
                atomic_maybe_uninit::cfg_has_atomic_128! {
                    atomic!(@check, $t, atomic_maybe_uninit::AtomicMaybeUninit<u128>, $a, $atomic_op);
                }
            }

            break $fallback_op;
        }
    };
}

macro_rules! impl_arithmetic {
	($t:ty, fetch_update, $example:tt) => {
		impl AtomicCell<$t> {
			/// Increments the current value by `val` and returns the previous value.
			///
			/// The addition wraps on overflow.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_add(3), 7);
			/// assert_eq!(a.load(), 10);
			/// ```
			#[inline]
			pub fn fetch_add(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(old.wrapping_add(val))).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = value.wrapping_add(val);
						old
					}
				}
			}

			/// Decrements the current value by `val` and returns the previous value.
			///
			/// The subtraction wraps on overflow.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_sub(3), 7);
			/// assert_eq!(a.load(), 4);
			/// ```
			#[inline]
			pub fn fetch_sub(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(old.wrapping_sub(val))).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = value.wrapping_sub(val);
						old
					}
				}
			}

			/// Applies bitwise "and" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_and(3), 7);
			/// assert_eq!(a.load(), 3);
			/// ```
			#[inline]
			pub fn fetch_and(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(old & val)).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value &= val;
						old
					}
				}
			}

			/// Applies bitwise "nand" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_nand(3), 7);
			/// assert_eq!(a.load(), !(7 & 3));
			/// ```
			#[inline]
			pub fn fetch_nand(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(!(old & val))).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = !(old & val);
						old
					}
				}
			}

			/// Applies bitwise "or" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_or(16), 7);
			/// assert_eq!(a.load(), 23);
			/// ```
			#[inline]
			pub fn fetch_or(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(old | val)).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value |= val;
						old
					}
				}
			}

			/// Applies bitwise "xor" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_xor(2), 7);
			/// assert_eq!(a.load(), 5);
			/// ```
			#[inline]
			pub fn fetch_xor(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(old ^ val)).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value ^= val;
						old
					}
				}
			}

			/// Compares and sets the maximum of the current value and `val`,
			/// and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_max(2), 7);
			/// assert_eq!(a.load(), 7);
			/// ```
			#[inline]
			pub fn fetch_max(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(cmp::max(old, val))).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = cmp::max(old, val);
						old
					}
				}
			}

			/// Compares and sets the minimum of the current value and `val`,
			/// and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_min(2), 7);
			/// assert_eq!(a.load(), 2);
			/// ```
			#[inline]
			pub fn fetch_min(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						self.fetch_update(|old| Some(cmp::min(old, val))).unwrap()
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = cmp::min(old, val);
						old
					}
				}
			}
		}
	};
	($t:ty, $atomic:ident, $example:tt) => {
		impl AtomicCell<$t> {
			/// Increments the current value by `val` and returns the previous value.
			///
			/// The addition wraps on overflow.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_add(3), 7);
			/// assert_eq!(a.load(), 10);
			/// ```
			#[inline]
			pub fn fetch_add(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_add(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = value.wrapping_add(val);
						old
					}
				}
			}

			/// Decrements the current value by `val` and returns the previous value.
			///
			/// The subtraction wraps on overflow.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_sub(3), 7);
			/// assert_eq!(a.load(), 4);
			/// ```
			#[inline]
			pub fn fetch_sub(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_sub(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = value.wrapping_sub(val);
						old
					}
				}
			}

			/// Applies bitwise "and" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_and(3), 7);
			/// assert_eq!(a.load(), 3);
			/// ```
			#[inline]
			pub fn fetch_and(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_and(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value &= val;
						old
					}
				}
			}

			/// Applies bitwise "nand" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_nand(3), 7);
			/// assert_eq!(a.load(), !(7 & 3));
			/// ```
			#[inline]
			pub fn fetch_nand(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_nand(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = !(old & val);
						old
					}
				}
			}

			/// Applies bitwise "or" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_or(16), 7);
			/// assert_eq!(a.load(), 23);
			/// ```
			#[inline]
			pub fn fetch_or(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_or(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value |= val;
						old
					}
				}
			}

			/// Applies bitwise "xor" to the current value and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_xor(2), 7);
			/// assert_eq!(a.load(), 5);
			/// ```
			#[inline]
			pub fn fetch_xor(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_xor(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value ^= val;
						old
					}
				}
			}

			/// Compares and sets the maximum of the current value and `val`,
			/// and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_max(9), 7);
			/// assert_eq!(a.load(), 9);
			/// ```
			#[inline]
			pub fn fetch_max(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_max(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = cmp::max(old, val);
						old
					}
				}
			}

			/// Compares and sets the minimum of the current value and `val`,
			/// and returns the previous value.
			///
			/// # Examples
			///
			/// ```
			/// use crossbeam_utils::atomic::AtomicCell;
			///
			#[doc = $example]
			///
			/// assert_eq!(a.fetch_min(2), 7);
			/// assert_eq!(a.load(), 2);
			/// ```
			#[inline]
			pub fn fetch_min(&self, val: $t) -> $t {
				atomic! {
					$t, _a,
					{
						let a = unsafe { &*(self.as_ptr() as *const atomic::$atomic) };
						a.fetch_min(val, Ordering::AcqRel)
					},
					{
						let _guard = lock(self.as_ptr() as usize).write();
						let value = unsafe { &mut *(self.as_ptr()) };
						let old = *value;
						*value = cmp::min(old, val);
						old
					}
				}
			}
		}
	};
}

#[repr(transparent)]
pub struct AtomicCell<T> {
	value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> AtomicCell<T> {
	pub const fn new(value: T) -> Self {
		Self {
			value: UnsafeCell::new(MaybeUninit::new(value)),
		}
	}

	#[allow(clippy::missing_const_for_fn)]
	pub fn into_inner(self) -> T {
		let this = ManuallyDrop::new(self);

		unsafe { this.as_ptr().read() }
	}

	#[must_use]
	pub const fn is_lock_free() -> bool {
		atomic_is_lock_free::<T>()
	}

	pub fn store(&self, value: T) {
		if mem::needs_drop::<T>() {
			drop(self.swap(value));
		} else {
			unsafe {
				atomic_store(self.as_ptr(), value);
			}
		}
	}

	pub fn swap(&self, value: T) -> T {
		unsafe { atomic_swap(self.as_ptr(), value) }
	}

	pub const fn as_ptr(&self) -> *mut T {
		self.value.get().cast::<T>()
	}
}

impl<T: Copy> AtomicCell<T> {
	pub fn load(&self) -> T {
		unsafe { atomic_load(self.as_ptr()) }
	}
}

impl<T: Default> AtomicCell<T> {
	pub fn take(&self) -> T {
		self.swap(T::default())
	}
}

impl<T> AtomicCell<T>
where
	T: Copy + Eq,
{
	pub fn compare_exchange(&self, current: T, new: T) -> Result<T, T> {
		unsafe { atomic_compare_exchange_weak(self.as_ptr(), current, new) }
	}

	pub fn fetch_update(&self, mut f: impl FnMut(T) -> Option<T>) -> Result<T, T> {
		let mut prev = self.load();
		while let Some(next) = f(prev) {
			match self.compare_exchange(prev, next) {
				x @ Ok(_) => return x,
				Err(next_prev) => prev = next_prev,
			}
		}

		Err(prev)
	}
}

impl AtomicCell<bool> {
	pub fn fetch_and(&self, val: bool) -> bool {
		atomic! {
			bool, _a,
			{
				let a = unsafe { &*(self.as_ptr() as *const atomic::AtomicBool) };
				a.fetch_and(val, Ordering::AcqRel)
			},
			{
				let _guard = lock(self.as_ptr() as usize).write();
				let value = unsafe { &mut *(self.as_ptr()) };
				let old = *value;
				*value &= val;
				old
			}
		}
	}

	pub fn fetch_nand(&self, val: bool) -> bool {
		atomic! {
			bool, _a,
			{
				let a = unsafe { &*(self.as_ptr() as *const atomic::AtomicBool) };
				a.fetch_nand(val, Ordering::AcqRel)
			},
			{
				let _guard = lock(self.as_ptr() as usize).write();
				let value = unsafe { &mut *(self.as_ptr()) };
				let old = *value;
				*value = !(old & val);
				old
			}
		}
	}

	pub fn fetch_or(&self, val: bool) -> bool {
		atomic! {
			bool, _a,
			{
				let a = unsafe { &*(self.as_ptr() as *const atomic::AtomicBool) };
				a.fetch_or(val, Ordering::AcqRel)
			},
			{
				let _guard = lock(self.as_ptr() as usize).write();
				let value = unsafe { &mut *(self.as_ptr()) };
				let old = *value;
				*value |= val;
				old
			}
		}
	}

	pub fn fetch_xor(&self, val: bool) -> bool {
		atomic! {
			bool, _a,
			{
				let a = unsafe { &*(self.as_ptr() as *const atomic::AtomicBool) };
				a.fetch_xor(val, Ordering::AcqRel)
			},
			{
				let _guard = lock(self.as_ptr() as usize).write();
				let value = unsafe { &mut *(self.as_ptr()) };
				let old = *value;
				*value ^= val;
				old
			}
		}
	}
}

impl<T> Debug for AtomicCell<T>
where
	T: Copy + Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("AtomicCell")
			.field("value", &self.load())
			.finish()
	}
}

impl<T: Default> Default for AtomicCell<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}

impl<T> Drop for AtomicCell<T> {
	fn drop(&mut self) {
		if mem::needs_drop::<T>() {
			unsafe {
				self.as_ptr().drop_in_place();
			}
		}
	}
}

impl<T> From<T> for AtomicCell<T> {
	fn from(value: T) -> Self {
		Self::new(value)
	}
}

impl<T> RefUnwindSafe for AtomicCell<T> {}

unsafe impl<T: Send> Send for AtomicCell<T> {}
unsafe impl<T: Sync> Sync for AtomicCell<T> {}

impl<T> UnwindSafe for AtomicCell<T> {}

impl_arithmetic!(u8, AtomicU8, "let a = AtomicCell::new(7u8);");
impl_arithmetic!(i8, AtomicI8, "let a = AtomicCell::new(7i8);");
impl_arithmetic!(u16, AtomicU16, "let a = AtomicCell::new(7u16);");
impl_arithmetic!(i16, AtomicI16, "let a = AtomicCell::new(7i16);");

#[cfg(target_has_atomic = "32")]
impl_arithmetic!(u32, AtomicU32, "let a = AtomicCell::new(7u32);");
#[cfg(target_has_atomic = "32")]
impl_arithmetic!(i32, AtomicI32, "let a = AtomicCell::new(7i32);");
#[cfg(not(target_has_atomic = "32"))]
impl_arithmetic!(u32, fetch_update, "let a = AtomicCell::new(7u32);");
#[cfg(not(target_has_atomic = "32"))]
impl_arithmetic!(i32, fetch_update, "let a = AtomicCell::new(7i32);");

#[cfg(target_has_atomic = "64")]
impl_arithmetic!(u64, AtomicU64, "let a = AtomicCell::new(7u64);");
#[cfg(target_has_atomic = "64")]
impl_arithmetic!(i64, AtomicI64, "let a = AtomicCell::new(7i64);");
#[cfg(not(target_has_atomic = "64"))]
impl_arithmetic!(u64, fetch_update, "let a = AtomicCell::new(7u64);");
#[cfg(not(target_has_atomic = "64"))]
impl_arithmetic!(i64, fetch_update, "let a = AtomicCell::new(7i64);");

impl_arithmetic!(u128, fetch_update, "let a = AtomicCell::new(7u128);");
impl_arithmetic!(i128, fetch_update, "let a = AtomicCell::new(7i128);");

impl_arithmetic!(usize, AtomicUsize, "let a = AtomicCell::new(7usize);");
impl_arithmetic!(isize, AtomicIsize, "let a = AtomicCell::new(7isize);");

struct AtomicUnit;

#[allow(clippy::unused_self, clippy::missing_const_for_fn)]
impl AtomicUnit {
	fn load(&self, _: Ordering) {}

	fn store(&self, (): (), _: Ordering) {}

	fn swap(&self, (): (), _: Ordering) {}

	fn compare_exchange_weak(&self, (): (), (): (), _: Ordering, _: Ordering) -> Result<(), ()> {
		Ok(())
	}
}

const fn can_transmute<A, B>() -> bool {
	(mem::size_of::<A>() == mem::size_of::<B>()) & (mem::align_of::<A>() >= mem::align_of::<B>())
}

unsafe fn atomic_store<T>(dst: *mut T, value: T) {
	atomic! {
		T, a,
		{
			a = unsafe { &*(dst as *const _ as *const _) };
			a.store(unsafe { mem::transmute_copy(&value) }, Ordering::Release);
			mem::forget(value);
		},
		{
			let _guard = lock(dst as usize).write();
			unsafe { ptr::write(dst, value) }
		}
	}
}

unsafe fn atomic_swap<T>(dst: *mut T, value: T) -> T {
	atomic! {
		T, a,
		{
			a = unsafe { &*(dst as *const _ as *const _) };
			let res = unsafe { mem::transmute_copy(&a.swap(mem::transmute_copy(&value), Ordering::AcqRel))};
			mem::forget(value);
			res
		},
		{
			let _guard = lock(dst as usize).write();
			unsafe { ptr::replace(dst, value) }
		}
	}
}

#[expect(clippy::declare_interior_mutable_const)]
fn lock(addr: usize) -> &'static SeqLock {
	const LEN: usize = 67;
	const L: CachePadded<SeqLock> = CachePadded::new(SeqLock::new());
	static LOCKS: [CachePadded<SeqLock>; LEN] = [L; LEN];

	&LOCKS[addr % LEN]
}

#[allow(clippy::let_unit_value, clippy::ignored_unit_patterns)]
unsafe fn atomic_compare_exchange_weak<T>(dst: *mut T, mut current: T, new: T) -> Result<T, T>
where
	T: Copy + Eq,
{
	atomic! {
		T, a,
		{
			a = unsafe { &*(dst as *const _ as *const _) };
			let mut current_raw = unsafe { mem::transmute_copy(&current) };
			let new_raw = unsafe { mem::transmute_copy(&new) };

			loop {
				match a.compare_exchange_weak(current_raw, new_raw, Ordering::AcqRel, Ordering::Acquire) {
					Ok(_) => break Ok(current),
					Err(previous_raw) => {
						let previous = unsafe { mem::transmute_copy(&previous_raw) };

						if !T::eq(&previous, &current) {
							break Err(previous)
						}

						current = previous;
						current_raw = previous_raw;
					}
				}
			}
		},
		{
			let guard = lock(dst as usize).write();

			let old = unsafe { ptr::read(dst) };
			if T::eq(&old, &current) {
				unsafe { ptr::write(dst, new) }
				Ok(old)
			} else {
				guard.abort();
				Err(old)
			}
		}
	}
}

unsafe fn atomic_load<T: Copy>(src: *mut T) -> T {
	atomic! {
		T, a,
		{
			a = unsafe { &*(src as *const _ as *const _) };
			unsafe { mem::transmute_copy(&a.load(Ordering::Acquire)) }
		},
		{
			let lock = lock(src as usize);

			if let Some(stamp) = lock.optimistic_read() {
				let value = unsafe { ptr::read_volatile(src.cast::<MaybeUninit<T>>()) };
				if lock.validate_read(stamp) {
					return unsafe { value.assume_init() };
				}
			}

			let guard = lock.write();
			let value = unsafe { ptr::read(src) };
			guard.abort();
			value
		}
	}
}

const fn atomic_is_lock_free<T>() -> bool {
	atomic! { T, _a, true, false }
}
