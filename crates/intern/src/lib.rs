#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};
use core::{
	cmp::Ordering,
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	marker::PhantomData,
};

use hashbrown::HashMap;

pub struct Interned<T: ?Sized> {
	id: u32,
	marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Interned<T> {
	pub(crate) const fn from_id(id: u32) -> Self {
		Self {
			id,
			marker: PhantomData,
		}
	}

	pub(crate) const fn id(&self) -> u32 {
		self.id
	}
}

impl<T: ?Sized> Debug for Interned<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("I").field(&self.id).finish()
	}
}

impl<T: ?Sized> Eq for Interned<T> {}

impl<T: ?Sized> Hash for Interned<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl<T: ?Sized> Ord for Interned<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.id.cmp(&other.id)
	}
}

impl<T: ?Sized> PartialEq for Interned<T> {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
	}
}

impl<T: ?Sized> PartialOrd for Interned<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug)]
pub struct Interner<T: ?Sized> {
	vec: Vec<Rc<T>>,
	map: HashMap<Rc<T>, u32>,
	references: usize,
}

impl<T: ?Sized> Interner<T> {
	#[must_use]
	pub fn new() -> Self {
		Self {
			vec: Vec::new(),
			map: HashMap::new(),
			references: 0,
		}
	}
}

impl<T: ?Sized> Default for Interner<T> {
	fn default() -> Self {
		Self::new()
	}
}
