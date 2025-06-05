use std::thread;

use vmm_intern::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TestStruct(String, u64);

#[test]
fn basic_hash() {
	let interner = HashInterner::<&str>::new();

	assert_eq!(interner.intern_sized("foo"), interner.intern_sized("foo"));
	assert_ne!(interner.intern_sized("foo"), interner.intern_sized("bar"));
	assert_eq!(interner.len(), 0);

	let interner = HashInterner::<String>::new();

	let interned1 = interner.intern_sized("foo".to_owned());
	{
		let interned2 = interner.intern_sized("foo".to_owned());
		let interned3 = interner.intern_sized("bar".to_owned());

		assert_eq!(interned2.ref_count(), 3);
		assert_eq!(interned3.ref_count(), 2);
		assert_eq!(interner.len(), 2);
	}

	assert_eq!(interner.len(), 1);

	drop(interner);
	assert_eq!(interned1.ref_count(), 1);
}

#[test]
fn basic_hash_unsized() {
	let interner = HashInterner::<str>::new();

	assert_eq!(interner.intern_ref("foo"), interner.intern_ref("foo"));
	assert_ne!(interner.intern_ref("foo"), interner.intern_ref("bar"));
	assert_eq!(interner.len(), 0);

	let interned1 = interner.intern_ref("foo");
	{
		let interned2 = interner.intern_ref("foo");
		let interned3 = interner.intern_ref("bar");

		assert_eq!(interned2.ref_count(), 3);
		assert_eq!(interned3.ref_count(), 2);
		assert_eq!(interner.len(), 2);
	}

	assert_eq!(interner.len(), 1);

	assert_eq!(
		&raw const *interned1,
		&raw const *interner.intern_ref("foo")
	);

	drop(interner);

	assert_ne!(
		&raw const *interned1,
		&raw const *HashInterner::new().intern_ref("foo")
	);

	assert_eq!(interned1.ref_count(), 1);
}

#[test]
fn basic_ord() {
	let interner = OrdInterner::<&str>::new();

	assert_eq!(interner.intern_sized("foo"), interner.intern_sized("foo"));
	assert_ne!(interner.intern_sized("foo"), interner.intern_sized("bar"));

	assert_eq!(interner.len(), 0);

	let interner = OrdInterner::<String>::new();

	let interned1 = interner.intern_sized("foo".to_owned());
	{
		let interned2 = interner.intern_sized("foo".to_owned());
		let interned3 = interner.intern_sized("bar".to_owned());

		assert_eq!(interned2.ref_count(), 3);
		assert_eq!(interned3.ref_count(), 2);
		assert_eq!(interner.len(), 2);
	}

	assert_eq!(interner.len(), 1);

	drop(interner);
	assert_eq!(interned1.ref_count(), 1);
}

#[test]
fn basic_ord_unsized() {
	let interner = OrdInterner::<str>::new();

	assert_eq!(interner.intern_ref("foo"), interner.intern_ref("foo"));
	assert_ne!(interner.intern_ref("foo"), interner.intern_ref("bar"));

	assert_eq!(interner.len(), 0);

	let interned1 = interner.intern_ref("foo");
	{
		let interned2 = interner.intern_ref("foo");
		let interned3 = interner.intern_ref("bar");

		assert_eq!(interned2.ref_count(), 3);
		assert_eq!(interned3.ref_count(), 2);

		assert_eq!(interner.len(), 2);
	}

	assert_eq!(interner.len(), 1);

	assert_eq!(
		&raw const *interned1,
		&raw const *interner.intern_ref("foo")
	);

	drop(interner);

	assert_ne!(
		&raw const *interned1,
		&raw const *OrdInterner::new().intern_ref("foo")
	);

	assert_eq!(interned1.ref_count(), 1);
}

#[test]
fn sorting() {
	let interner = HashInterner::new();
	let mut interned_vals = [
		interner.intern_sized(4),
		interner.intern_sized(2),
		interner.intern_sized(5),
		interner.intern_sized(0),
		interner.intern_sized(1),
		interner.intern_sized(3),
	];

	interned_vals.sort();
	let sorted: Vec<String> = interned_vals.iter().map(|v| format!("{v}")).collect();
	assert_eq!(sorted.join(","), "0,1,2,3,4,5");
}

#[test]
#[expect(clippy::collection_is_never_read)]
fn sequential_hash() {
	let interner = HashInterner::new();

	for _ in 0..10 {
		let mut interned = Vec::with_capacity(100);
		for j in 0..10 {
			interned.push(interner.intern_sized(TestStruct("foo".to_owned(), j)));
		}
	}

	assert_eq!(interner.len(), 0);
}

#[test]
#[expect(clippy::collection_is_never_read)]
fn sequential_ord() {
	let interner = OrdInterner::new();

	for _ in 0..10 {
		let mut interned = Vec::with_capacity(100);
		for j in 0..10 {
			interned.push(interner.intern_sized(TestStruct("foo".to_owned(), j)));
		}
	}

	assert_eq!(interner.len(), 0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn multithreading_hash() -> thread::Result<()> {
	let interner = HashInterner::new();
	let mut thandles = Vec::new();
	for _ in 0..3 {
		let interner = interner.clone();
		thandles.push(thread::spawn(move || {
			for _ in 0..10 {
				let _interned1 = interner.intern_sized(TestStruct("foo".to_owned(), 5));
				let _interned2 = interner.intern_sized(TestStruct("bar".to_owned(), 10));
			}
		}));
	}

	for h in thandles {
		h.join()?;
	}

	assert_eq!(interner.len(), 0);

	Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn multithreading_ord() -> thread::Result<()> {
	let interner = OrdInterner::new();

	let mut thandles = Vec::new();
	for _ in 0..3 {
		let interner = interner.clone();
		thandles.push(thread::spawn(move || {
			for _ in 0..10 {
				let _interned1 = interner.intern_sized(TestStruct("foo".to_owned(), 5));
				let _interned2 = interner.intern_sized(TestStruct("bar".to_owned(), 10));
			}
		}));
	}

	for h in thandles {
		h.join()?;
	}

	assert_eq!(interner.len(), 0);

	Ok(())
}
