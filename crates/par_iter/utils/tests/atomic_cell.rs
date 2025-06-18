use std::{
	mem,
	sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

use vmm_par_utils::atomic::AtomicCell;

const fn always_use_fallback() -> bool {
	atomic_maybe_uninit::cfg_has_atomic_cas! {
		cfg!(miri)
	}
	atomic_maybe_uninit::cfg_no_atomic_cas! { true }
}

#[test]
#[allow(dead_code)]
fn is_lock_free() {
	struct UsizeWrap(usize);
	struct U8Wrap(bool);
	struct I16Wrap(i16);
	#[repr(align(8))]
	struct U64Align8(u64);

	let always_use_fallback = always_use_fallback();

	assert_eq!(AtomicCell::<usize>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<isize>::is_lock_free(), !always_use_fallback);
	assert_eq!(
		AtomicCell::<UsizeWrap>::is_lock_free(),
		!always_use_fallback
	);

	assert!(AtomicCell::<()>::is_lock_free());

	assert_eq!(AtomicCell::<u8>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<i8>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<bool>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<U8Wrap>::is_lock_free(), !always_use_fallback);

	assert_eq!(AtomicCell::<u16>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<i16>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<I16Wrap>::is_lock_free(), !always_use_fallback);

	assert_eq!(AtomicCell::<u32>::is_lock_free(), !always_use_fallback);
	assert_eq!(AtomicCell::<i32>::is_lock_free(), !always_use_fallback);

	assert_eq!(
		AtomicCell::<u64>::is_lock_free(),
		cfg!(target_has_atomic = "64")
			&& matches!(std::mem::align_of::<u64>(), 8)
			&& !always_use_fallback
	);
	assert_eq!(mem::size_of::<U64Align8>(), 8);
	assert_eq!(mem::align_of::<U64Align8>(), 8);
	assert_eq!(
		AtomicCell::<U64Align8>::is_lock_free(),
		cfg!(target_has_atomic = "64") && !always_use_fallback
	);

	assert_eq!(
		AtomicCell::<u128>::is_lock_free(),
		cfg!(target_has_atomic = "128")
            // && std::mem::align_of::<u128>() == 16
            && matches!(std::mem::align_of::<u128>(), 16)
            && !always_use_fallback
	);
}

#[test]
fn const_is_lock_free() {
	const _U: bool = AtomicCell::<usize>::is_lock_free();
	const _I: bool = AtomicCell::<isize>::is_lock_free();
}

#[test]
fn drops_unit() {
	#[derive(Debug, PartialEq, Eq)]
	struct Foo();

	impl Foo {
		fn new() -> Self {
			CNT.fetch_add(1, SeqCst);
			Self()
		}
	}

	impl Default for Foo {
		fn default() -> Self {
			Self::new()
		}
	}

	impl Drop for Foo {
		fn drop(&mut self) {
			CNT.fetch_sub(1, SeqCst);
		}
	}

	static CNT: AtomicUsize = AtomicUsize::new(0);
	CNT.store(0, SeqCst);

	let a = AtomicCell::new(Foo::new());

	assert_eq!(a.swap(Foo::new()), Foo::new());
	assert_eq!(CNT.load(SeqCst), 1);

	a.store(Foo::new());
	assert_eq!(CNT.load(SeqCst), 1);

	assert_eq!(a.swap(Foo::default()), Foo::new());
	assert_eq!(CNT.load(SeqCst), 1);

	drop(a);
	assert_eq!(CNT.load(SeqCst), 0);
}

#[test]
fn drops_u8() {
	#[derive(Debug, PartialEq, Eq)]
	struct Foo(u8);

	impl Foo {
		fn new(value: u8) -> Self {
			CNT.fetch_add(1, SeqCst);
			Self(value)
		}
	}

	impl Default for Foo {
		fn default() -> Self {
			Self::new(0)
		}
	}

	impl Drop for Foo {
		fn drop(&mut self) {
			CNT.fetch_sub(1, SeqCst);
		}
	}

	static CNT: AtomicUsize = AtomicUsize::new(0);
	CNT.store(0, SeqCst);

	let a = AtomicCell::new(Foo::new(5));

	assert_eq!(a.swap(Foo::new(6)), Foo::new(5));
	assert_eq!(a.swap(Foo::new(1)), Foo::new(6));
	assert_eq!(CNT.load(SeqCst), 1);

	a.store(Foo::new(2));
	assert_eq!(CNT.load(SeqCst), 1);

	assert_eq!(a.swap(Foo::default()), Foo::new(2));
	assert_eq!(CNT.load(SeqCst), 1);

	assert_eq!(a.swap(Foo::default()), Foo::new(0));
	assert_eq!(CNT.load(SeqCst), 1);

	drop(a);
	assert_eq!(CNT.load(SeqCst), 0);
}

#[test]
fn drops_usize() {
	#[derive(Debug, PartialEq, Eq)]
	struct Foo(usize);

	impl Foo {
		fn new(value: usize) -> Self {
			CNT.fetch_add(1, SeqCst);
			Self(value)
		}
	}

	impl Default for Foo {
		fn default() -> Self {
			Self::new(0)
		}
	}

	impl Drop for Foo {
		fn drop(&mut self) {
			CNT.fetch_sub(1, SeqCst);
		}
	}

	static CNT: AtomicUsize = AtomicUsize::new(0);
	CNT.store(0, SeqCst);

	let a = AtomicCell::new(Foo::new(5));

	assert_eq!(a.swap(Foo::new(6)), Foo::new(5));
	assert_eq!(a.swap(Foo::new(1)), Foo::new(6));
	assert_eq!(CNT.load(SeqCst), 1);

	a.store(Foo::new(2));
	assert_eq!(CNT.load(SeqCst), 1);

	assert_eq!(a.swap(Foo::default()), Foo::new(2));
	assert_eq!(CNT.load(SeqCst), 1);

	assert_eq!(a.swap(Foo::default()), Foo::new(0));
	assert_eq!(CNT.load(SeqCst), 1);

	drop(a);
	assert_eq!(CNT.load(SeqCst), 0);
}

#[test]
fn modular_u8() {
	#[derive(Clone, Copy, Eq, Debug, Default)]
	struct Foo(u8);

	impl PartialEq for Foo {
		fn eq(&self, other: &Self) -> bool {
			self.0 % 5 == other.0 % 5
		}
	}

	let a = AtomicCell::new(Foo(1));

	assert_eq!(a.load(), Foo(1));
	assert_eq!(a.swap(Foo(2)), Foo(11));
	assert_eq!(a.load(), Foo(52));

	a.store(Foo(0));
	assert_eq!(a.compare_exchange(Foo(0), Foo(5)), Ok(Foo(100)));
	assert_eq!(a.load().0, 5);
	assert_eq!(a.compare_exchange(Foo(10), Foo(15)), Ok(Foo(100)));
	assert_eq!(a.load().0, 15);
}

#[test]
fn modular_usize() {
	#[derive(Clone, Copy, Eq, Debug, Default)]
	struct Foo(usize);

	impl PartialEq for Foo {
		fn eq(&self, other: &Self) -> bool {
			self.0 % 5 == other.0 % 5
		}
	}

	let a = AtomicCell::new(Foo(1));

	assert_eq!(a.load(), Foo(1));
	assert_eq!(a.swap(Foo(2)), Foo(11));
	assert_eq!(a.load(), Foo(52));

	a.store(Foo(0));
	assert_eq!(a.compare_exchange(Foo(0), Foo(5)), Ok(Foo(100)));
	assert_eq!(a.load().0, 5);
	assert_eq!(a.compare_exchange(Foo(10), Foo(15)), Ok(Foo(100)));
	assert_eq!(a.load().0, 15);
}

#[test]
#[allow(clippy::no_effect_underscore_binding)]
fn garbage_padding() {
	#[derive(Copy, Clone, Eq, PartialEq)]
	struct Object {
		a: i64,
		b: i32,
	}

	let cell = AtomicCell::new(Object { a: 0, b: 0 });
	let _garbage = [0xfe, 0xfe, 0xfe, 0xfe, 0xfe]; // Needed
	let next = Object { a: 0, b: 0 };

	let prev = cell.load();
	assert!(cell.compare_exchange(prev, next).is_ok());
	println!();
}

#[test]
fn const_atomic_cell_new() {
	static CELL: AtomicCell<usize> = AtomicCell::new(0);

	CELL.store(1);
	assert_eq!(CELL.load(), 1);
}

#[test]
fn doesnt_access_uninit() {
	#[repr(align(8))]
	#[allow(dead_code)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	enum Test {
		Field(u32),
		Fieldless,
	}

	assert_eq!(mem::size_of::<Test>(), 8);
	assert_eq!(
		AtomicCell::<Test>::is_lock_free(),
		cfg!(target_has_atomic = "64") && !always_use_fallback()
	);
	let x = AtomicCell::new(Test::Fieldless);
	assert_eq!(x.load(), Test::Fieldless);
}

#[test]
fn works_with_niches() {
	use std::{
		num::NonZeroU128,
		sync::atomic::{AtomicBool, Ordering},
		thread,
	};

	#[allow(dead_code)]
	enum Enum {
		NeverConstructed,
		Cell(AtomicCell<NonZeroU128>),
	}

	static STATE: Enum = Enum::Cell(AtomicCell::new(match NonZeroU128::new(1) {
		Some(nonzero) => nonzero,
		None => unreachable!(),
	}));
	static FINISHED: AtomicBool = AtomicBool::new(false);

	const N: usize = if cfg!(miri) { 10_000 } else { 1_000_000 };

	let handle = thread::spawn(|| {
		let cell = match &STATE {
			Enum::NeverConstructed => unreachable!(),
			Enum::Cell(cell) => cell,
		};

		let x = NonZeroU128::new(0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000).unwrap();
		let y = NonZeroU128::new(0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF).unwrap();
		while !FINISHED.load(Ordering::Relaxed) {
			cell.store(x);
			cell.store(y);
		}
	});

	for _ in 0..N {
		if matches!(STATE, Enum::NeverConstructed) {
			unreachable!(":(");
		}
	}

	FINISHED.store(true, Ordering::Relaxed);
	handle.join().unwrap();
}
