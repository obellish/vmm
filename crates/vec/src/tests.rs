#![allow(clippy::from_iter_instead_of_collect, clippy::while_let_on_iterator)]

extern crate std;

use alloc::{
	borrow::{Borrow, BorrowMut, ToOwned},
	boxed::Box,
	rc::Rc,
	vec,
	vec::Vec,
};
use core::{
	cell::Cell,
	hash::{Hash, Hasher},
	iter::FromIterator,
	ptr,
};

use super::SmallVec;

#[test]
fn zero() {
	let mut v = SmallVec::<_, 0>::new();
	assert!(!v.spilled());
	v.push(0usize);
	assert!(v.spilled());
	assert_eq!(&*v, [0]);
}

#[test]
fn inline() {
	let mut v = SmallVec::<_, 16>::new();
	v.push("hello".to_owned());
	v.push("there".to_owned());
	assert_eq!(&*v, ["hello", "there"]);
}

#[test]
fn spill() {
	let mut v = SmallVec::<_, 2>::new();
	v.push("hello".to_owned());
	assert_eq!(v[0], "hello");
	v.push("there".to_owned());
	v.push("burma".to_owned());
	assert_eq!(v[0], "hello");
	v.push("shave".to_owned());
	assert_eq!(&*v, ["hello", "there", "burma", "shave"]);
}

#[test]
fn double_spill() {
	let mut v = SmallVec::<_, 2>::new();
	v.push("hello".to_owned());
	v.push("there".to_owned());
	v.push("burma".to_owned());
	v.push("shave".to_owned());
	v.push("hello".to_owned());
	v.push("there".to_owned());
	v.push("burma".to_owned());
	v.push("shave".to_owned());

	assert_eq!(
		&*v,
		[
			"hello", "there", "burma", "shave", "hello", "there", "burma", "shave"
		]
	);
}

#[test]
fn with_capacity() {
	let v = SmallVec::<u8, 3>::with_capacity(1);
	assert!(v.is_empty());
	assert!(!v.spilled());
	assert_eq!(v.capacity(), 3);

	let v = SmallVec::<u8, 3>::with_capacity(10);
	assert!(v.is_empty());
	assert!(v.spilled());
	assert_eq!(v.capacity(), 10);
}

#[test]
fn drain() {
	let mut v = SmallVec::<u8, 2>::new();
	v.push(3);
	assert_eq!(v.drain(..).collect::<Vec<_>>(), [3]);

	v.push(3);
	v.push(4);
	v.push(5);
	let old_capacity = v.capacity();
	assert_eq!(v.drain(1..).collect::<Vec<_>>(), [4, 5]);
	assert_eq!(v.capacity(), old_capacity);

	let mut v = SmallVec::<u8, 2>::new();
	v.push(1);
	v.push(2);
	assert_eq!(v.drain(..1).collect::<Vec<_>>(), [1]);
}

#[test]
fn drain_rev() {
	let mut v = SmallVec::<u8, 2>::new();
	v.push(3);
	assert_eq!(v.drain(..).rev().collect::<Vec<_>>(), [3]);

	v.push(3);
	v.push(4);
	v.push(5);
	assert_eq!(v.drain(..).rev().collect::<Vec<_>>(), [5, 4, 3]);
}

#[test]
fn drain_forget() {
	let mut v = SmallVec::<u8, 1>::from([0, 1, 2, 3, 4, 5, 6, 7]);
	core::mem::forget(v.drain(2..5));
	assert_eq!(v.len(), 2);
}

#[test]
fn splice() {
	let mut v = SmallVec::<u8, 1>::from([0, 1, 2, 3, 4, 5, 6]);
	let new = [7, 8, 9, 10];
	let u = v.splice(6.., new).collect::<SmallVec<u8, 1>>();
	assert_eq!(v, [0, 1, 2, 3, 4, 5, 7, 8, 9, 10]);
	assert_eq!(u, [6]);

	let mut v = SmallVec::<u8, 1>::from([0, 1, 2, 3, 4, 5, 6]);
	let new = [7, 8, 9, 10];
	let u = v.splice(1..1, new).collect::<SmallVec<u8, 1>>();
	assert_eq!(v, [0, 7, 8, 9, 10, 1, 2, 3, 4, 5, 6]);
	assert_eq!(u, []);

	let mut v = SmallVec::<u8, 1>::from([0, 1, 2, 3, 4, 5, 6]);
	let new = [7, 8, 9, 10];
	let u = v.splice(..3, new).collect::<SmallVec<u8, 1>>();
	assert_eq!(v, [7, 8, 9, 10, 3, 4, 5, 6]);
	assert_eq!(u, [0, 1, 2]);
}

#[test]
fn into_iter() {
	let mut v = SmallVec::<u8, 2>::new();
	v.push(3);
	assert_eq!(v.into_iter().collect::<Vec<_>>(), [3]);

	let mut v = SmallVec::<u8, 2>::new();
	v.push(3);
	v.push(4);
	v.push(5);
	assert_eq!(v.into_iter().collect::<Vec<_>>(), [3, 4, 5]);
}

#[test]
fn into_iter_rev() {
	let mut v = SmallVec::<u8, 2>::new();
	v.push(3);
	assert_eq!(v.into_iter().rev().collect::<Vec<_>>(), [3]);

	let mut v = SmallVec::<u8, 2>::new();
	v.push(3);
	v.push(4);
	v.push(5);
	assert_eq!(v.into_iter().rev().collect::<Vec<_>>(), [5, 4, 3]);
}

#[test]
fn into_iter_drop() {
	struct DropCounter<'a>(&'a Cell<i32>);

	impl Drop for DropCounter<'_> {
		fn drop(&mut self) {
			self.0.set(self.0.get() + 1);
		}
	}

	{
		let cell = Cell::new(0);
		let mut v = SmallVec::<_, 2>::new();
		v.push(DropCounter(&cell));
		v.into_iter();
		assert_eq!(cell.get(), 1);
	}

	{
		let cell = Cell::new(0);
		let mut v = SmallVec::<_, 2>::new();
		v.push(DropCounter(&cell));
		v.push(DropCounter(&cell));
		assert!(v.into_iter().next().is_some());
		assert_eq!(cell.get(), 2);
	}

	{
		let cell = Cell::new(0);
		let mut v = SmallVec::<_, 2>::new();
		v.push(DropCounter(&cell));
		v.push(DropCounter(&cell));
		v.push(DropCounter(&cell));
		{
			let mut it = v.into_iter();
			assert!(it.next().is_some());
			assert!(it.next_back().is_some());
		}
		assert_eq!(cell.get(), 3);
	}
}

#[test]
fn capacity() {
	let mut v = SmallVec::<u8, 2>::new();
	v.reserve(1);
	assert_eq!(v.capacity(), 2);
	assert!(!v.spilled());

	v.reserve_exact(0x100);
	assert!(v.capacity() >= 0x100);

	v.push(0);
	v.push(1);
	v.push(2);
	v.push(3);

	v.shrink_to_fit();

	assert!(v.capacity() < 0x100);
}

#[test]
fn truncate() {
	let mut v = SmallVec::<_, 8>::new();

	for x in 0u8..8 {
		v.push(Box::new(x));
	}

	v.truncate(4);

	assert_eq!(v.len(), 4);
	assert!(!v.spilled());

	assert_eq!(*v.swap_remove(1), 1);
	assert_eq!(*v.remove(1), 3);
	v.insert(1, Box::new(3));

	assert_eq!(v.iter().map(|v| **v).collect::<Vec<_>>(), [0u8, 3, 2]);
}

#[test]
fn truncate_references() {
	let mut v = [0, 1, 2, 3, 4, 5, 6, 7];
	let mut i = 8;
	let mut v: SmallVec<&mut u8, 8> = v.iter_mut().collect();

	v.truncate(4);
	assert!(!v.spilled());

	assert_eq!(*v.swap_remove(1), 1);
	assert_eq!(*v.remove(1), 3);
	v.insert(1, &mut i);

	assert_eq!(
		v.iter_mut().map(|v| &mut **v).collect::<Vec<_>>(),
		[&mut 0, &mut 8, &mut 2]
	);
}

#[test]
fn split_off() {
	let mut vec = SmallVec::<u32, 4>::from([1, 2, 3, 4, 5, 6]);
	let orig_ptr = vec.as_ptr();
	let orig_capacity = vec.capacity();

	let split_off = vec.split_off(4);
	assert_eq!(&vec[..], [1, 2, 3, 4]);
	assert_eq!(&split_off[..], [5, 6]);
	assert_eq!(vec.capacity(), orig_capacity);
	assert!(ptr::eq(vec.as_ptr(), orig_ptr));
}

#[test]
fn split_all_off() {
	let mut vec = SmallVec::<u32, 4>::with_capacity(1000);
	vec.extend([1, 2, 3, 4, 5, 6]);
	let orig_ptr = vec.as_ptr();
	let orig_capacity = vec.capacity();

	let split_off = vec.split_off(0);
	assert_eq!(&vec[..], []);
	assert_eq!(&split_off[..], [1, 2, 3, 4, 5, 6]);
	assert_eq!(vec.capacity(), orig_capacity);
	assert!(ptr::eq(vec.as_ptr(), orig_ptr));

	assert!(split_off.capacity() < orig_capacity);
	assert!(!ptr::eq(split_off.as_ptr(), orig_ptr));
}

#[test]
fn append() {
	let mut v = SmallVec::<u8, 8>::new();
	for x in 0..4 {
		v.push(x);
	}

	assert_eq!(v.len(), 4);

	let mut n = SmallVec::<u8, 2>::from_buf([5, 6]);
	v.append(&mut n);
	assert_eq!(v.len(), 6);
	assert_eq!(n.len(), 0);

	assert_eq!(v.iter().copied().collect::<Vec<_>>(), [0, 1, 2, 3, 5, 6]);
}

#[test]
#[should_panic(expected = "assertion failed: new_capacity >= len")]
fn invalid_grow() {
	let mut v = SmallVec::<u8, 8>::new();
	v.extend(0..8);
	v.grow(5);
}

#[test]
#[should_panic(expected = "range end out of bounds")]
fn drain_overflow() {
	let mut v = SmallVec::<u8, 8>::from([0]);
	v.drain(..=usize::MAX);
}

#[test]
fn insert_from_slice() {
	let mut v = SmallVec::<u8, 8>::new();
	for x in 0..4 {
		v.push(x);
	}

	assert_eq!(v.len(), 4);
	v.insert_from_slice(1, &[5, 6]);
	assert_eq!(v.iter().copied().collect::<Vec<_>>(), [0, 5, 6, 1, 2, 3]);
}

#[test]
fn extend_from_slice() {
	let mut v = SmallVec::<u8, 8>::new();
	for x in 0..4 {
		v.push(x);
	}

	assert_eq!(v.len(), 4);
	v.extend_from_slice(&[5, 6]);
	assert_eq!(v.iter().copied().collect::<Vec<_>>(), [0, 1, 2, 3, 5, 6]);
}

#[test]
#[should_panic(expected = "drop")]
fn drop_panic_smallvec() {
	struct DropPanic;

	impl Drop for DropPanic {
		fn drop(&mut self) {
			panic!("drop");
		}
	}

	let mut v = SmallVec::<_, 1>::new();
	v.push(DropPanic);
}

#[test]
fn eq() {
	let mut a = SmallVec::<u32, 2>::new();
	let mut b = SmallVec::<u32, 2>::new();
	let mut c = SmallVec::<u32, 2>::new();

	a.push(1);
	a.push(2);

	b.push(1);
	b.push(2);

	c.push(3);
	c.push(4);

	assert_eq!(a, b);
	assert_ne!(a, c);
}

#[test]
fn ord() {
	let mut a = SmallVec::<u32, 2>::new();
	let mut b = SmallVec::<u32, 2>::new();
	let mut c = SmallVec::<u32, 2>::new();

	a.push(1);

	b.push(1);
	b.push(1);

	c.push(1);
	c.push(2);

	assert!(a < b);
	assert!(b > a);
	assert!(b < c);
	assert!(c > b);
}

#[test]
fn hash() {
	use std::collections::hash_map::DefaultHasher;

	fn hash(value: impl Hash) -> u64 {
		let mut hasher = DefaultHasher::new();
		value.hash(&mut hasher);
		hasher.finish()
	}

	{
		let mut a = SmallVec::<u32, 2>::new();
		let b = [1, 2];
		a.extend(b.iter().copied());
		assert_eq!(hash(a), hash(b));
	}

	{
		let mut a = SmallVec::<u32, 2>::new();
		let b = [1, 2, 11, 12];
		a.extend(b.iter().copied());
		assert_eq!(hash(a), hash(b));
	}
}

#[test]
fn as_ref() {
	let mut a = SmallVec::<u32, 2>::new();
	a.push(1);
	assert_eq!(a.as_ref(), [1]);
	a.push(2);
	assert_eq!(a.as_ref(), [1, 2]);
	a.push(3);
	assert_eq!(a.as_ref(), [1, 2, 3]);
}

#[test]
fn as_mut() {
	let mut a = SmallVec::<u32, 2>::new();
	a.push(1);
	assert_eq!(a.as_mut(), [1]);
	a.push(2);
	assert_eq!(a.as_mut(), [1, 2]);
	a.push(3);
	assert_eq!(a.as_mut(), [1, 2, 3]);
	a.as_mut()[1] = 4;
	assert_eq!(a.as_mut(), [1, 4, 3]);
}

#[test]
fn borrow() {
	let mut a = SmallVec::<u32, 2>::new();
	a.push(1);
	assert_eq!(a.borrow(), [1]);
	a.push(2);
	assert_eq!(a.borrow(), [1, 2]);
	a.push(3);
	assert_eq!(a.borrow(), [1, 2, 3]);
}

#[test]
fn borrow_mut() {
	let mut a = SmallVec::<u32, 2>::new();
	a.push(1);
	assert_eq!(a.borrow_mut(), [1]);
	a.push(2);
	assert_eq!(a.borrow_mut(), [1, 2]);
	a.push(3);
	assert_eq!(a.borrow_mut(), [1, 2, 3]);
	BorrowMut::<[u32]>::borrow_mut(&mut a)[1] = 4;
	assert_eq!(a.borrow_mut(), [1, 4, 3]);
}

#[test]
fn from() {
	#[derive(Debug, PartialEq, Eq)]
	struct NoClone(u8);

	assert_eq!(&SmallVec::<u32, 2>::from(&[1][..])[..], [1]);
	assert_eq!(&SmallVec::<u32, 2>::from(&[1, 2, 3][..])[..], [1, 2, 3]);

	let vec = Vec::new();
	let small_vec = SmallVec::<u8, 3>::from(vec);
	assert_eq!(&*small_vec, []);
	drop(small_vec);

	let vec = vec![1, 2, 3, 4, 5];
	let small_vec = SmallVec::<u8, 3>::from(vec);
	assert_eq!(&*small_vec, [1, 2, 3, 4, 5]);
	drop(small_vec);

	let array = [1];
	let small_vec = SmallVec::<u8, 1>::from(array);
	assert_eq!(&*small_vec, [1]);
	drop(small_vec);

	let array = [99; 128];
	let small_vec = SmallVec::<u8, 128>::from(array);
	assert_eq!(&*small_vec, [99u8; 128]);
	drop(small_vec);

	let array = [NoClone(42)];
	let small_vec = SmallVec::<_, 1>::from(array);
	assert_eq!(&*small_vec, [NoClone(42)]);
	drop(small_vec);

	let array = [1; 128];
	let small_vec = SmallVec::<u8, 1>::from(array);
	assert_eq!(&*small_vec, [1; 128]);
	drop(small_vec);

	let array = [99];
	let small_vec = SmallVec::<u8, 128>::from(array);
	assert_eq!(&*small_vec, [99]);
	drop(small_vec);
}

#[test]
fn from_slice() {
	assert_eq!(&SmallVec::<u32, 2>::from_slice(&[1][..])[..], [1]);
	assert_eq!(
		&SmallVec::<u32, 2>::from_slice(&[1, 2, 3][..])[..],
		[1, 2, 3]
	);
}

#[test]
fn exact_size_iterator() {
	let mut vec = SmallVec::<u32, 2>::from(&[1, 2, 3][..]);
	assert_eq!(vec.clone().into_iter().len(), 3);
	assert_eq!(vec.drain(..2).len(), 2);
	assert_eq!(vec.into_iter().len(), 1);
}

#[test]
#[allow(clippy::redundant_clone)]
fn into_iter_as_slice() {
	let vec = SmallVec::<u32, 2>::from(&[1, 2, 3][..]);
	let mut iter = vec.clone().into_iter();
	assert_eq!(iter.as_slice(), [1, 2, 3]);
	assert_eq!(iter.as_mut_slice(), [1, 2, 3]);
	iter.next();
	assert_eq!(iter.as_slice(), [2, 3]);
	assert_eq!(iter.as_mut_slice(), [2, 3]);
	iter.next_back();
	assert_eq!(iter.as_slice(), [2]);
	assert_eq!(iter.as_mut_slice(), [2]);
}

#[test]
fn into_iter_clone() {
	let mut iter = SmallVec::<u8, 2>::from_iter(0..3).into_iter();
	let mut clone_iter = iter.clone();
	while let Some(x) = iter.next() {
		assert_eq!(x, clone_iter.next().unwrap());
	}

	assert_eq!(clone_iter.next(), None);
}

#[test]
fn into_iter_clone_partially_consumed_iterator() {
	let mut iter = SmallVec::<u8, 2>::from_iter(0..3).into_iter().skip(1);
	let mut clone_iter = iter.clone();
	while let Some(x) = iter.next() {
		assert_eq!(x, clone_iter.next().unwrap());
	}

	assert_eq!(clone_iter.next(), None);
}

#[test]
fn into_iter_empty_smallvec() {
	let mut iter = SmallVec::<u8, 2>::new().into_iter();
	let mut clone_iter = iter.clone();
	assert_eq!(iter.next(), None);
	assert_eq!(clone_iter.next(), None);
}

#[test]
fn shrink_to_fit_unspill() {
	let mut vec = SmallVec::<u8, 2>::from_iter(0..3);
	vec.pop();
	assert!(vec.spilled());
	vec.shrink_to_fit();
	assert!(!vec.spilled());
}

#[test]
fn shrink_after_from_empty_vec() {
	let mut v = SmallVec::<u8, 2>::from_vec(Vec::new());
	v.shrink_to_fit();
	assert!(!v.spilled());
}

#[test]
fn into_vec() {
	let vec = SmallVec::<u8, 2>::from_iter(0..2);
	assert_eq!(vec.into_vec(), vec![0, 1]);

	let vec = SmallVec::<u8, 2>::from_iter(0..3);
	assert_eq!(vec.into_vec(), vec![0, 1, 2]);
}

#[test]
fn into_inner() {
	let vec = SmallVec::<u8, 2>::from_iter(0..2);
	assert_eq!(vec.into_inner(), Ok([0, 1]));

	let vec = SmallVec::<u8, 2>::from_iter(0..1);
	assert_eq!(vec.clone().into_inner(), Err(vec));

	let vec = SmallVec::<u8, 2>::from_iter(0..3);
	assert_eq!(vec.clone().into_inner(), Err(vec));
}

#[test]
fn from_vec() {
	let vec = Vec::new();
	let small_vec = SmallVec::<u8, 3>::from_vec(vec);
	assert_eq!(&*small_vec, []);
	drop(small_vec);

	let vec = Vec::new();
	let small_vec = SmallVec::<u8, 1>::from_vec(vec);
	assert_eq!(&*small_vec, []);
	drop(small_vec);

	let vec = vec![1];
	let small_vec = SmallVec::<u8, 3>::from_vec(vec);
	assert_eq!(&*small_vec, [1]);
	drop(small_vec);

	let vec = vec![1, 2, 3];
	let small_vec = SmallVec::<u8, 3>::from_vec(vec);
	assert_eq!(&*small_vec, [1, 2, 3]);
	drop(small_vec);

	let vec = vec![1, 2, 3, 4, 5];
	let small_vec = SmallVec::<u8, 3>::from_vec(vec);
	assert_eq!(&*small_vec, [1, 2, 3, 4, 5]);

	let vec = vec![1, 2, 3, 4, 5];
	let small_vec = SmallVec::<u8, 1>::from_vec(vec);
	assert_eq!(&*small_vec, [1, 2, 3, 4, 5]);
	drop(small_vec);
}

#[test]
fn retain() {
	let mut sv = SmallVec::<i32, 5>::from_slice(&[1, 2, 3, 3, 4]);
	sv.retain(|&i| !matches!(i, 3));
	assert_eq!(sv.pop(), Some(4));
	assert_eq!(sv.pop(), Some(2));
	assert_eq!(sv.pop(), Some(1));
	assert_eq!(sv.pop(), None);

	let mut sv = SmallVec::<i32, 3>::from_slice(&[1, 2, 3, 3, 4]);
	sv.retain(|&i| !matches!(i, 3));
	assert_eq!(sv.pop(), Some(4));
	assert_eq!(sv.pop(), Some(2));
	assert_eq!(sv.pop(), Some(1));
	assert_eq!(sv.pop(), None);

	let one = Rc::new(1);
	let mut sv = SmallVec::<_, 3>::new();
	sv.push(Rc::clone(&one));
	assert_eq!(Rc::strong_count(&one), 2);
	sv.retain(|_| false);
	assert_eq!(Rc::strong_count(&one), 1);

	let mut sv = SmallVec::<_, 1>::new();
	sv.push(Rc::clone(&one));
	sv.push(Rc::new(2));
	assert_eq!(Rc::strong_count(&one), 2);
	sv.retain(|_| false);
	assert_eq!(Rc::strong_count(&one), 1);
}

#[test]
fn dedup() {
	let mut dupes = SmallVec::<i32, 5>::from_slice(&[1, 1, 2, 3, 3]);
	dupes.dedup();
	assert_eq!(&*dupes, [1, 2, 3]);

	let mut empty = SmallVec::<i32, 5>::new();
	empty.dedup();
	assert!(empty.is_empty());

	let mut all_ones = SmallVec::<i32, 5>::from_slice(&[1, 1, 1, 1, 1]);
	all_ones.dedup();
	assert_eq!(all_ones.len(), 1);

	let mut no_dupes = SmallVec::<i32, 5>::from_slice(&[1, 2, 3, 4, 5]);
	no_dupes.dedup();
	assert_eq!(no_dupes.len(), 5);
}

#[test]
fn resize() {
	let mut v = SmallVec::<i32, 8>::new();
	v.push(1);
	v.resize(5, 0);
	assert_eq!(&*v, [1, 0, 0, 0, 0]);

	v.resize(2, -1);
	assert_eq!(&*v, [1, 0]);
}

#[test]
#[cfg(feature = "std")]
fn write() -> std::io::Result<()> {
	use std::io::Write;

	let data = [1, 2, 3, 4, 5];

	let mut small_vec = SmallVec::<u8, 2>::new();
	let len = small_vec.write(&data[..])?;
	assert_eq!(len, 5);
	assert_eq!(small_vec.as_ref(), data.as_ref());

	Ok(())
}

#[test]
fn grow_to_shrink() {
	let mut v = SmallVec::<u8, 2>::new();
	v.push(1);
	v.push(2);
	v.push(3);
	assert!(v.spilled());
	v.clear();

	v.grow(2);
	assert!(!v.spilled());
	assert_eq!(v.capacity(), 2);
	assert_eq!(v.len(), 0);
	v.push(4);
	assert_eq!(&*v, [4]);
}

#[test]
fn resumable_extend() {
	let s = "a b c";
	let it = s
		.chars()
		.scan(0, |_, ch| if ch.is_whitespace() { None } else { Some(ch) });

	let mut v = SmallVec::<char, 4>::new();
	v.extend(it);
	assert_eq!(v, ['a']);
}

#[test]
fn uninhabited() {
	enum Void {}

	let _sv = SmallVec::<Void, 8>::new();
}

#[test]
fn grow_spilled_same_size() {
	let mut v = SmallVec::<u8, 2>::new();
	v.push(0);
	v.push(1);
	v.push(2);
	assert!(v.spilled());
	assert_eq!(v.capacity(), 4);

	v.grow(4);
	assert_eq!(v.capacity(), 4);
	assert_eq!(v, [0, 1, 2]);
}

#[test]
fn const_generics() {
	let _v = SmallVec::<i32, 987>::default();
}

#[test]
fn const_new() {
	const fn const_new_inner() -> SmallVec<i32, 4> {
		SmallVec::new()
	}

	let v = const_new_inner();
	assert_eq!(v.capacity(), 4);
	assert_eq!(v.len(), 0);
}

#[test]
fn zero_size_items() {
	SmallVec::<(), 0>::new().push(());
}

#[test]
fn clone_from() {
	let mut a = SmallVec::<u8, 2>::new();
	a.push(1);
	a.push(2);
	a.push(3);

	let mut b = SmallVec::<u8, 2>::new();
	b.push(10);

	let mut c = SmallVec::<u8, 2>::new();
	c.push(20);
	c.push(21);
	c.push(22);

	a.clone_from(&b);
	assert_eq!(a, [10]);

	b.clone_from(&c);
	assert_eq!(b, [20, 21, 22]);
}

#[test]
fn extract_if() {
	let mut a = SmallVec::<u8, 2>::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 0]);

	let b: SmallVec<u8, 2> = a.extract_if(1..9, |x| matches!(*x % 3, 0)).collect();

	assert_eq!(a, SmallVec::<u8, 2>::from_slice(&[0, 1, 2, 4, 5, 7, 8, 0]));
	assert_eq!(b, SmallVec::<u8, 2>::from_slice(&[3, 6]));
}

#[test]
fn max_dont_panic() {
	let mut sv = SmallVec::<i32, 2>::from([0]);
	let _ = sv.get(usize::MAX);
	sv.truncate(usize::MAX);
}

#[test]
#[should_panic(expected = "removal index (is 18446744073709551615) should be < len (is 1)")]
fn max_remove() {
	let mut sv = SmallVec::<i32, 2>::from([0]);
	sv.remove(usize::MAX);
}

#[test]
#[should_panic(expected = "swap_remove index (is 18446744073709551615) should be < len (is 1)")]
fn max_swap_remove() {
	let mut sv = SmallVec::<i32, 2>::from([0]);
	sv.swap_remove(usize::MAX);
}

#[test]
#[should_panic(expected = "insertion index (is 18446744073709551615) should be <= len (is 1)")]
fn max_insert() {
	let mut sv = SmallVec::<i32, 2>::from([0]);
	sv.insert(usize::MAX, 0);
}

#[test]
fn collect_from_iter() {
	#[repr(transparent)]
	struct IterNoHint<I: Iterator>(I);

	impl<I: Iterator> Iterator for IterNoHint<I> {
		type Item = I::Item;

		fn next(&mut self) -> Option<Self::Item> {
			self.0.next()
		}
	}

	let iter = IterNoHint(core::iter::repeat_n(
		1u8,
		if cfg!(miri) { 3 } else { 1_000_000 },
	));

	let _y = SmallVec::<_, 1>::from_iter(iter);
}

#[test]
fn collect_with_spill() {
	let input = "0123456";
	let collected: SmallVec<char, 4> = input.chars().collect();
	assert_eq!(collected, ['0', '1', '2', '3', '4', '5', '6']);
}

#[test]
fn spare_capacity_mut() {
	let mut v = SmallVec::<u8, 2>::new();
	assert!(!v.spilled());
	let spare = v.spare_capacity_mut();
	assert_eq!(spare.len(), 2);
	assert!(ptr::eq(spare.as_ptr().cast::<u8>(), v.as_ptr()));

	v.push(1);
	assert!(!v.spilled());
	let spare = v.spare_capacity_mut();
	assert_eq!(spare.len(), 1);
	assert!(ptr::eq(spare.as_ptr().cast::<u8>(), unsafe {
		v.as_ptr().add(1)
	}));

	v.push(2);
	assert!(!v.spilled());
	let spare = v.spare_capacity_mut();
	assert_eq!(spare.len(), 0);
	assert!(ptr::eq(spare.as_ptr().cast::<u8>(), unsafe {
		v.as_ptr().add(2)
	}));

	v.push(3);
	assert!(v.spilled());
	let spare = v.spare_capacity_mut();
	assert!(!spare.is_empty());
	assert!(ptr::eq(spare.as_ptr().cast::<u8>(), unsafe {
		v.as_ptr().add(3)
	}));
}
