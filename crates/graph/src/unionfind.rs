use alloc::{collections::TryReserveError, vec, vec::Vec};
use core::cmp::Ordering;

use super::graph::IndexType;

#[derive(Debug, Clone)]
pub struct UnionFind<K> {
	parent: Vec<K>,
	rank: Vec<u8>,
}

impl<K> UnionFind<K> {
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			parent: Vec::with_capacity(capacity),
			rank: Vec::with_capacity(capacity),
		}
	}

	#[must_use]
	pub fn capacity(&self) -> usize {
		self.parent.capacity().min(self.rank.capacity())
	}

	pub fn reserve(&mut self, additional: usize) {
		self.parent.reserve(additional);
		self.rank.reserve(additional);
	}

	pub fn reserve_exact(&mut self, additional: usize) {
		self.parent.reserve_exact(additional);
		self.rank.reserve_exact(additional);
	}

	pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.parent
			.try_reserve(additional)
			.and_then(|()| self.rank.try_reserve(additional))
	}

	pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.parent
			.try_reserve_exact(additional)
			.and_then(|()| self.rank.try_reserve_exact(additional))
	}

	pub fn shrink_to_fit(&mut self) {
		self.parent.shrink_to_fit();
		self.rank.shrink_to_fit();
	}

	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.parent.shrink_to(min_capacity);
		self.rank.shrink_to(min_capacity);
	}
}

impl<K: IndexType> UnionFind<K> {
	pub fn new(n: usize) -> Self {
		let rank = vec![0; n];
		let parent = (0..n).map(K::new).collect();

		Self { parent, rank }
	}

	#[must_use]
	pub const fn empty() -> Self {
		Self {
			parent: Vec::new(),
			rank: Vec::new(),
		}
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.parent.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.parent.is_empty()
	}

	pub fn add_set(&mut self) -> K {
		let retval = K::new(self.parent.len());
		self.rank.push(0);
		self.parent.push(retval);
		retval
	}

	#[track_caller]
	pub fn find(&self, x: K) -> K {
		self.try_find(x).expect("the index is out of bounds")
	}

	pub fn try_find(&self, mut x: K) -> Option<K> {
		if x.index() >= self.len() {
			return None;
		}

		loop {
			let xparent = unsafe { *get_unchecked(&self.parent, x.index()) };
			if xparent == x {
				break Some(x);
			}

			x = xparent;
		}
	}

	#[track_caller]
	pub fn find_mut(&mut self, x: K) -> K {
		assert!(x.index() < self.len());
		unsafe { self.find_mut_recursive(x) }
	}

	pub fn try_find_mut(&mut self, x: K) -> Option<K> {
		if x.index() >= self.len() {
			return None;
		}

		Some(unsafe { self.find_mut_recursive(x) })
	}

	unsafe fn find_mut_recursive(&mut self, mut x: K) -> K {
		let mut parent = *unsafe { get_unchecked(&self.parent, x.index()) };
		while parent != x {
			let grandparent = *unsafe { get_unchecked(&self.parent, parent.index()) };
			*unsafe { get_unchecked_mut(&mut self.parent, x.index()) } = grandparent;
			x = parent;
			parent = grandparent;
		}

		x
	}

	#[track_caller]
	pub fn equiv(&self, x: K, y: K) -> bool {
		self.find(x) == self.find(y)
	}

	pub fn try_equiv(&self, x: K, y: K) -> Result<bool, K> {
		let xrep = self.try_find(x).ok_or(x)?;
		let yrep = self.try_find(y).ok_or(y)?;
		Ok(xrep == yrep)
	}

	#[track_caller]
	pub fn union(&mut self, x: K, y: K) -> bool {
		self.try_union(x, y).unwrap()
	}

	pub fn try_union(&mut self, x: K, y: K) -> Result<bool, K> {
		if x == y {
			return Ok(false);
		}

		let xrep = self.try_find_mut(x).ok_or(x)?;
		let yrep = self.try_find_mut(y).ok_or(y)?;

		if xrep == yrep {
			return Ok(false);
		}

		let xrepu = xrep.index();
		let yrepu = yrep.index();
		let xrank = self.rank[xrepu];
		let yrank = self.rank[yrepu];

		match xrank.cmp(&yrank) {
			Ordering::Less => self.parent[xrepu] = yrep,
			Ordering::Greater => self.parent[yrepu] = xrep,
			Ordering::Equal => {
				self.parent[yrepu] = xrep;
				self.rank[xrepu] += 1;
			}
		}

		Ok(true)
	}

	#[must_use]
	pub fn into_labeling(mut self) -> Vec<K> {
		unsafe {
			for ix in 0..self.len() {
				let k = *get_unchecked(&self.parent, ix);
				let xrep = self.find_mut_recursive(k);
				*self.parent.get_unchecked_mut(ix) = xrep;
			}
		}

		self.parent
	}
}

impl<K> Default for UnionFind<K> {
	fn default() -> Self {
		Self {
			parent: Vec::new(),
			rank: Vec::new(),
		}
	}
}

unsafe fn get_unchecked<K>(xs: &[K], index: usize) -> &K {
	debug_assert!(index < xs.len());
	unsafe { xs.get_unchecked(index) }
}

unsafe fn get_unchecked_mut<K>(xs: &mut [K], index: usize) -> &mut K {
	debug_assert!(index < xs.len());
	unsafe { xs.get_unchecked_mut(index) }
}
