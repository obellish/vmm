use std::{
	borrow::Borrow,
	cmp::{self, Ordering},
	collections::BTreeSet,
	fmt::{Debug, Display, Formatter, Pointer, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::Deref,
	sync::Arc,
};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};

use super::ref_count::{Interned, Interner};

#[repr(transparent)]
pub struct OrdInterner<T>
where
	T: ?Sized + cmp::Ord,
{
	inner: Arc<Ord<T>>,
}

impl<T> OrdInterner<T>
where
	T: ?Sized + cmp::Ord,
{
	#[must_use]
	pub fn new() -> Self {
		Self {
			inner: Arc::new(Ord {
				set: RwLock::new(BTreeSet::new()),
			}),
		}
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.inner.set.read().len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.inner.set.read().is_empty()
	}

	fn intern<U>(&self, value: U, intern: impl FnOnce(U) -> InternedOrd<T>) -> InternedOrd<T>
	where
		U: Borrow<T>,
	{
		let set = self.inner.set.upgradable_read();
		if let Some(entry) = set.get(value.borrow()) {
			return entry.clone();
		}
		let mut set = RwLockUpgradableReadGuard::upgrade(set);
		if let Some(entry) = set.get(value.borrow()) {
			return entry.clone();
		}
		let mut ret = intern(value);
		ret.0.make_hot(&self.inner);
		set.insert(ret.clone());
		ret
	}

	pub fn intern_ref(&self, value: &T) -> InternedOrd<T>
	where
		T: ToOwned,
		T::Owned: Into<Box<T>>,
	{
		self.intern(value, |v| {
			InternedOrd(Interned::from_box(v.to_owned().into()))
		})
	}

	#[must_use]
	pub fn intern_box(&self, value: Box<T>) -> InternedOrd<T> {
		self.intern(value, |v| InternedOrd(Interned::from_box(v)))
	}

	pub fn intern_sized(&self, value: T) -> InternedOrd<T>
	where
		T: Sized,
	{
		self.intern(value, |v| InternedOrd(Interned::from_sized(v)))
	}
}

impl<T> Clone for OrdInterner<T>
where
	T: ?Sized + cmp::Ord,
{
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

impl<T> Default for OrdInterner<T>
where
	T: ?Sized + cmp::Ord,
{
	fn default() -> Self {
		Self::new()
	}
}

#[repr(C)]
pub struct Ord<T>
where
	T: ?Sized + cmp::Ord,
{
	set: RwLock<BTreeSet<InternedOrd<T>>>,
}

impl<T> Interner for Ord<T>
where
	T: ?Sized + cmp::Ord,
{
	type T = T;

	fn remove(&self, value: &Interned<Self>) -> (bool, Option<Interned<Self>>) {
		let value = cast(value);
		let mut set = self.set.write();
		if let Some(i) = set.take(value) {
			if matches!(i.ref_count(), 1) {
				(true, Some(i.0))
			} else {
				set.insert(i);
				(false, None)
			}
		} else {
			(true, None)
		}
	}
}

unsafe impl<T> Send for Ord<T> where T: ?Sized + cmp::Ord + Send + Sync {}
unsafe impl<T> Sync for Ord<T> where T: ?Sized + cmp::Ord + Send + Sync {}

#[repr(transparent)]
pub struct InternedOrd<T>(Interned<Ord<T>>)
where
	T: ?Sized + cmp::Ord;

impl<T> InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	#[must_use]
	pub fn ref_count(&self) -> u32 {
		self.0.ref_count()
	}
}

impl<T> AsRef<T> for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T> Borrow<T> for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	fn borrow(&self) -> &T {
		self
	}
}

impl<T> Clone for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> Debug for InternedOrd<T>
where
	T: ?Sized + Debug + cmp::Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("Interned").field(&&**self).finish()
	}
}

impl<T> Deref for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> Display for InternedOrd<T>
where
	T: ?Sized + Display + cmp::Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&&**self, f)
	}
}

impl<T> Eq for InternedOrd<T> where T: ?Sized + cmp::Ord {}

impl<T> Hash for InternedOrd<T>
where
	T: ?Sized + Hash + cmp::Ord,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.deref().hash(state);
	}
}

impl<T> cmp::Ord for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	fn cmp(&self, other: &Self) -> Ordering {
		if self.0 == other.0 {
			return Ordering::Equal;
		}

		self.deref().cmp(&**other)
	}
}

impl<T> PartialEq for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 || self.deref().eq(&**other)
	}
}

impl<T> PartialOrd for InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<T> Pointer for InternedOrd<T>
where
	T: ?Sized + Display + cmp::Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Pointer::fmt(&(&raw const **self), f)
	}
}

const fn cast<T>(i: &Interned<Ord<T>>) -> &InternedOrd<T>
where
	T: ?Sized + cmp::Ord,
{
	unsafe { &*std::ptr::from_ref(i).cast::<InternedOrd<T>>() }
}

#[cfg(test)]
mod tests {
	#[test]
	fn size() {
		let s = std::mem::size_of::<super::Ord<()>>();
		assert!(s < 100, "too big: {s}");
	}
}
