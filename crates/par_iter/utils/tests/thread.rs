use std::{
	any::Any,
	sync::atomic::{AtomicUsize, Ordering},
	thread::sleep,
	time::Duration,
};

use vmm_par_utils::thread;

const THREADS: usize = 10;
const SMALL_STACK_SIZE: usize = 20;

#[test]
fn join() {
	let counter = AtomicUsize::new(0);
	thread::scope(|scope| {
		let handle = scope.spawn(|_| {
			counter.store(1, Ordering::Relaxed);
		});

		assert!(handle.join().is_ok());

		let panic_handle = scope.spawn(|_| {
			panic!();
		});

		assert!(panic_handle.join().is_err());
	})
	.unwrap();

	assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[test]
fn counter() {
	let counter = AtomicUsize::new(0);

	thread::scope(|scope| {
		for _ in 0..THREADS {
			scope.spawn(|_| {
				counter.fetch_add(1, Ordering::Relaxed);
			});
		}
	})
	.unwrap();

	assert_eq!(counter.load(Ordering::Relaxed), THREADS);
}

#[test]
fn counter_builder() {
	let counter = AtomicUsize::new(0);
	thread::scope(|scope| {
		for i in 0..THREADS {
			scope
				.builder()
				.name(format!("child-{i}"))
				.stack_size(SMALL_STACK_SIZE)
				.spawn(|_| counter.fetch_add(1, Ordering::Relaxed))
				.unwrap();
		}
	})
	.unwrap();

	assert_eq!(counter.load(Ordering::Relaxed), THREADS);
}

#[test]
fn counter_panic() {
	let counter = AtomicUsize::new(0);
	let result = thread::scope(|scope| {
		scope.spawn(|_| panic!());
		sleep(Duration::from_millis(100));

		for _ in 0..THREADS {
			scope.spawn(|_| {
				counter.fetch_add(1, Ordering::Relaxed);
			});
		}
	});

	assert_eq!(counter.load(Ordering::Relaxed), THREADS);
	assert!(result.is_err());
}

#[test]
fn panic_twice() {
	let result = thread::scope(|scope| {
		scope.spawn(|_| {
			sleep(Duration::from_millis(500));
			panic!("thread #1");
		});

		scope.spawn(|_| {
			panic!("thread #2");
		});
	});

	let err = result.unwrap_err();
	let vec = err
		.downcast_ref::<Vec<Box<dyn Any + Send + 'static>>>()
		.unwrap();

	assert_eq!(vec.len(), 2);

	let first = vec[0].downcast_ref::<&str>().unwrap();
	let second = vec[1].downcast_ref::<&str>().unwrap();
	assert_eq!(*first, "thread #1");
	assert_eq!(*second, "thread #2");
}

#[test]
fn panic_many() {
	let result = thread::scope(|scope| {
		scope.spawn(|_| panic!("deliberate panic #1"));
		scope.spawn(|_| panic!("deliberate panic #2"));
		scope.spawn(|_| panic!("deliberate panic #3"));
	});

	let err = result.unwrap_err();
	let vec = err
		.downcast_ref::<Vec<Box<dyn Any + Send + 'static>>>()
		.unwrap();
	assert_eq!(vec.len(), 3);

	for (i, panic) in vec.iter().enumerate() {
		let panic = panic.downcast_ref::<&str>().unwrap();

		assert_eq!(*panic, format!("deliberate panic #{}", i + 1));
	}
}

#[test]
fn nesting() {
	struct Wrapper<'a> {
		var: &'a String,
	}

	impl<'a> Wrapper<'a> {
		fn recurse(&'a self, scope: &thread::Scope<'a>, depth: usize) {
			assert_eq!(self.var, "foo");

			if depth > 0 {
				scope.spawn(move |scope| {
					self.recurse(scope, depth - 1);
				});
			}
		}
	}

	let var = String::from("foo");

	let wrapper = Wrapper { var: &var };

	thread::scope(|scope| {
		scope.spawn(|scope| {
			scope.spawn(|scope| {
				wrapper.recurse(scope, 3);
			});
		});
	})
	.unwrap();
}

#[test]
fn join_nested() {
	thread::scope(|scope| {
		scope.spawn(|scope| {
			let handle = scope.spawn(|_| 7);

			sleep(Duration::from_millis(200));
			handle.join().unwrap()
		});

		sleep(Duration::from_millis(100));
	})
	.unwrap();
}

#[test]
fn scope_returns_ok() {
	let result = thread::scope(|scope| scope.spawn(|_| 1234).join().unwrap()).unwrap();

	assert_eq!(result, 1234);
}

#[test]
#[cfg(windows)]
fn as_raw_handle() {
	use std::os::windows::io::AsRawHandle as _;

	thread::scope(|scope| {
		let handle = scope.spawn(|_| {
			sleep(Duration::from_millis(100));
			42
		});

		let _raw_handle = handle.as_raw_handle();
		handle.join().unwrap();
	})
	.unwrap();
}
