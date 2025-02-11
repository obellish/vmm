#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod inner;
mod macros;

use alloc::{
	boxed::Box,
	collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
	rc::Rc,
	string::String,
	sync::Arc,
	vec::Vec,
};

pub use self::inner::{Length, SmallLength};

pub trait Len {
	fn length(&self) -> Length;
}

impl Len for Length {
	fn length(&self) -> Length {
		*self
	}
}

impl Len for SmallLength {
	fn length(&self) -> Length {
		(*self).into()
	}
}

impl<T> Len for [T] {
	fn length(&self) -> Length {
		self.len().into()
	}
}

impl Len for String {
	fn length(&self) -> Length {
		self.as_str().length()
	}
}

impl Len for str {
	fn length(&self) -> Length {
		self.len().into()
	}
}

impl<K, V> Len for BTreeMap<K, V> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

impl<V> Len for BTreeSet<V> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

impl<T> Len for Vec<T> {
	fn length(&self) -> Length {
		(**self).length()
	}
}

impl<T> Len for VecDeque<T> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

impl<T: Len> Len for Box<T> {
	fn length(&self) -> Length {
		(**self).length()
	}
}

impl<T: Len> Len for Arc<T> {
	fn length(&self) -> Length {
		(**self).length()
	}
}

impl<T: Len> Len for Rc<T> {
	fn length(&self) -> Length {
		(**self).length()
	}
}

impl<T> Len for BinaryHeap<T> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

#[cfg(feature = "bumpalo")]
impl Len for bumpalo::Bump {
	fn length(&self) -> Length {
		self.allocated_bytes().into()
	}
}

#[cfg(feature = "bytes")]
impl Len for bytes::Bytes {
	fn length(&self) -> Length {
		self.len().into()
	}
}

#[cfg(feature = "bytes")]
impl Len for bytes::BytesMut {
	fn length(&self) -> Length {
		self.len().into()
	}
}

#[cfg(feature = "indexmap")]
impl<K, V, S> Len for indexmap::IndexMap<K, V, S> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

#[cfg(feature = "indexmap")]
impl<V, S> Len for indexmap::IndexSet<V, S> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

#[cfg(feature = "std")]
impl<K, V, S> Len for std::collections::HashMap<K, V, S> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

#[cfg(feature = "std")]
impl<T, S> Len for std::collections::HashSet<T, S> {
	fn length(&self) -> Length {
		self.len().into()
	}
}

pub trait SmallLen: Len {
	fn small_len(&self) -> SmallLength {
		self.length().into()
	}
}

impl<T: Len> SmallLen for T {}
