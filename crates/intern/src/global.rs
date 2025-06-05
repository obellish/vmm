use std::{
	any::{Any, TypeId},
	collections::HashMap,
	hash::Hash,
	sync::LazyLock,
};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};

use super::{HashInterner, OrdInterner};

static HASH_INTERNERS: LazyLock<RwLock<HashInternerPool>> = LazyLock::new(Default::default);

#[repr(transparent)]
pub struct HashInternerPool {
	type_map: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
}

impl HashInternerPool {
	#[must_use]
	pub fn new() -> Self {
		Self {
			type_map: HashMap::new(),
		}
	}

	#[must_use]
	pub fn get<T>(&self) -> Option<HashInterner<T>>
	where
		T: ?Sized + Any + Eq + Hash + Send + Sync + 'static,
	{
		self.type_map
			.get(&TypeId::of::<T>())
			.and_then(|v| v.downcast_ref::<HashInterner<T>>())
			.cloned()
	}

	pub fn get_or_create<T>(&mut self) -> HashInterner<T>
	where
		T: ?Sized + Any + Eq + Hash + Send + Sync + 'static,
	{
		self.get::<T>().unwrap_or_else(|| {
			let ret = HashInterner::new();
			self.type_map
				.insert(TypeId::of::<T>(), Box::new(ret.clone()));
			ret
		})
	}
}

impl Default for HashInternerPool {
	fn default() -> Self {
		Self::new()
	}
}

static ORD_INTERNERS: LazyLock<RwLock<OrdInternerPool>> = LazyLock::new(Default::default);

#[repr(transparent)]
pub struct OrdInternerPool {
	type_map: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
}

impl OrdInternerPool {
	#[must_use]
	pub fn new() -> Self {
		Self {
			type_map: HashMap::new(),
		}
	}

	#[must_use]
	pub fn get<T>(&self) -> Option<OrdInterner<T>>
	where
		T: ?Sized + Any + Ord + Send + Sync + 'static,
	{
		self.type_map
			.get(&TypeId::of::<T>())
			.and_then(|v| v.downcast_ref())
			.cloned()
	}

	pub fn get_or_create<T>(&mut self) -> OrdInterner<T>
	where
		T: ?Sized + Any + Ord + Send + Sync + 'static,
	{
		self.get::<T>().unwrap_or_else(|| {
			let ret = OrdInterner::new();
			self.type_map
				.insert(TypeId::of::<T>(), Box::new(ret.clone()));
			ret
		})
	}
}

impl Default for OrdInternerPool {
	fn default() -> Self {
		Self::new()
	}
}

pub fn hash_interner<T>() -> HashInterner<T>
where
	T: ?Sized + Any + Eq + Hash + Send + Sync + 'static,
{
	let map = HASH_INTERNERS.upgradable_read();
	if let Some(interner) = map.get::<T>() {
		return interner;
	}

	let mut map = RwLockUpgradableReadGuard::upgrade(map);
	map.get_or_create()
}

pub fn ord_interner<T>() -> OrdInterner<T>
where
	T: ?Sized + Any + Ord + Send + Sync + 'static,
{
	let map = ORD_INTERNERS.upgradable_read();
	if let Some(interner) = map.get::<T>() {
		return interner;
	}

	let mut map = RwLockUpgradableReadGuard::upgrade(map);
	map.get_or_create()
}
