use std::{
	borrow::Borrow,
	cmp::Ordering,
	collections::HashSet,
	fmt::{Debug, Display, Formatter, Pointer, Result as FmtResult},
	hash::{self, Hasher},
	ops::Deref,
	sync::Arc,
};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};

use super::ref_count::{Interned, Interner};

#[repr(transparent)]
pub struct HashInterner<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	inner: Arc<Hash<T>>,
}

impl<T> HashInterner<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	#[must_use]
	pub fn new() -> Self {
		Self {
			inner: Arc::new(Hash {
				set: RwLock::new(HashSet::new()),
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

	fn intern<U>(&self, value: U, intern: impl FnOnce(U) -> InternedHash<T>) -> InternedHash<T>
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

	pub fn intern_ref(&self, value: &T) -> InternedHash<T>
	where
		T: ToOwned,
		T::Owned: Into<Box<T>>,
	{
		self.intern(value, |v| {
			InternedHash(Interned::from_box(v.to_owned().into()))
		})
	}

	#[must_use]
	pub fn intern_box(&self, value: Box<T>) -> InternedHash<T> {
		self.intern(value, |v| InternedHash(Interned::from_box(v)))
	}

	pub fn intern_sized(&self, value: T) -> InternedHash<T>
	where
		T: Sized,
	{
		self.intern(value, |v| InternedHash(Interned::from_sized(v)))
	}
}

impl<T> Default for HashInterner<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Clone for HashInterner<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

#[repr(C)]
pub struct Hash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	set: RwLock<HashSet<InternedHash<T>>>,
}

impl<T> Interner for Hash<T>
where
	T: ?Sized + Eq + hash::Hash,
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

unsafe impl<T> Send for Hash<T> where T: ?Sized + Eq + hash::Hash + Send + Sync {}
unsafe impl<T> Sync for Hash<T> where T: ?Sized + Eq + hash::Hash + Send + Sync {}

#[repr(transparent)]
pub struct InternedHash<T>(Interned<Hash<T>>)
where
	T: ?Sized + Eq + hash::Hash;

impl<T> InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	#[must_use]
	pub fn ref_count(&self) -> u32 {
		self.0.ref_count()
	}
}

impl<T> AsRef<T> for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T> Borrow<T> for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn borrow(&self) -> &T {
		self
	}
}

impl<T> Clone for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> Debug for InternedHash<T>
where
	T: ?Sized + Debug + Eq + hash::Hash,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("Interned").field(&&**self).finish()
	}
}

impl<T> Deref for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> Display for InternedHash<T>
where
	T: ?Sized + Display + Eq + hash::Hash,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&&**self, f)
	}
}

impl<T> Eq for InternedHash<T> where T: ?Sized + Eq + hash::Hash {}

impl<T> hash::Hash for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.deref().hash(state);
	}
}

impl<T> Ord for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash + Ord,
{
	fn cmp(&self, other: &Self) -> Ordering {
		if self.0 == other.0 {
			return Ordering::Equal;
		}

		self.deref().cmp(&**other)
	}
}

impl<T> PartialEq for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 || self.deref().eq(&**other)
	}
}

impl<T> PartialOrd for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash + PartialOrd,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		if self.0 == other.0 {
			return Some(Ordering::Equal);
		}

		self.deref().partial_cmp(&**other)
	}
}

impl<T> Pointer for InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Pointer::fmt(&(&raw const **self), f)
	}
}

const fn cast<T>(i: &Interned<Hash<T>>) -> &InternedHash<T>
where
	T: ?Sized + Eq + hash::Hash,
{
	unsafe { &*std::ptr::from_ref(i).cast::<InternedHash<T>>() }
}
