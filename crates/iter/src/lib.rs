#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod adapters;

use core::cmp::Ordering;

pub use self::adapters::*;

pub trait IteratorExt: Iterator {
	#[cfg(feature = "alloc")]
	fn sorted(self) -> Sorted<Self::Item>
	where
		Self: Sized,
		Self::Item: Ord,
	{
		Sorted::new(self)
	}

	#[cfg(feature = "alloc")]
	fn sorted_by<F>(self, sorter: F) -> SortedBy<Self::Item, F>
	where
		Self: Sized,
		F: FnMut(&Self::Item, &Self::Item) -> Ordering,
	{
		SortedBy::new(self, sorter)
	}

	fn sorted_by_key<K: Ord, F>(self, sorter: F) -> SortedByKey<Self::Item, K, F>
	where
		Self: Sized,
		F: FnMut(&Self::Item) -> K,
	{
		SortedByKey::new(self, sorter)
	}
}

impl<T> IteratorExt for T where T: Iterator {}
