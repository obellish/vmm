#[cfg(windows)]
pub mod windows;

use std::{
	boxed::Box,
	fmt::{Debug, Formatter, Result as FmtResult},
	io,
	marker::PhantomData,
	mem, panic,
	string::String,
	sync::{Arc, Mutex},
	thread,
	vec::Vec,
};

use super::sync::WaitGroup;

pub struct Scope<'env> {
	handles: SharedVec<SharedOption<thread::JoinHandle<()>>>,
	wait_group: WaitGroup,
	marker: PhantomData<&'env mut &'env ()>,
}

impl<'env> Scope<'env> {
	pub fn spawn<'scope, F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope, T>
	where
		F: FnOnce(&Self) -> T + Send + 'env,
		T: Send + 'env,
	{
		self.builder()
			.spawn(f)
			.expect("failed to spawn scoped thread")
	}

	pub fn builder<'scope>(&'scope self) -> ScopedThreadBuilder<'scope, 'env> {
		ScopedThreadBuilder {
			scope: self,
			builder: thread::Builder::new(),
		}
	}
}

impl Debug for Scope<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.pad("Scope { .. }")
	}
}

unsafe impl Sync for Scope<'_> {}

#[derive(Debug)]
#[must_use = "must eventually spawn the thread"]
pub struct ScopedThreadBuilder<'scope, 'env> {
	scope: &'scope Scope<'env>,
	builder: thread::Builder,
}

impl<'scope, 'env> ScopedThreadBuilder<'scope, 'env> {
	pub fn name(mut self, name: String) -> Self {
		self.builder = self.builder.name(name);
		self
	}

	pub fn stack_size(mut self, size: usize) -> Self {
		self.builder = self.builder.stack_size(size);
		self
	}

	pub fn spawn<F, T>(self, f: F) -> io::Result<ScopedJoinHandle<'scope, T>>
	where
		F: FnOnce(&Scope<'env>) -> T + Send + 'env,
		T: Send + 'env,
	{
		let result = SharedOption::default();

		let (handle, thread) = {
			let result = Arc::clone(&result);

			let scope = Scope::<'env> {
				handles: Arc::clone(&self.scope.handles),
				wait_group: self.scope.wait_group.clone(),
				marker: PhantomData,
			};

			let handle = {
				let closure = move || {
					let scope: Scope<'env> = scope;

					let res = f(&scope);

					*result.lock().unwrap() = Some(res);
				};

				let closure: Box<dyn FnOnce() + Send + 'env> = Box::new(closure);
				let closure: Box<dyn FnOnce() + Send + 'static> =
					unsafe { mem::transmute(closure) };

				self.builder.spawn(closure)?
			};

			let thread = handle.thread().clone();
			let handle = Arc::new(Mutex::new(Some(handle)));

			(handle, thread)
		};

		self.scope.handles.lock().unwrap().push(Arc::clone(&handle));

		Ok(ScopedJoinHandle {
			handle,
			result,
			thread,
			marker: PhantomData,
		})
	}
}

pub struct ScopedJoinHandle<'scope, T> {
	handle: SharedOption<thread::JoinHandle<()>>,
	result: SharedOption<T>,
	thread: thread::Thread,
	marker: PhantomData<&'scope ()>,
}

impl<T> ScopedJoinHandle<'_, T> {
	pub fn join(self) -> thread::Result<T> {
		let handle = self.handle.lock().unwrap().take().unwrap();

		handle
			.join()
			.map(|()| self.result.lock().unwrap().take().unwrap())
	}

	#[must_use]
	pub const fn thread(&self) -> &thread::Thread {
		&self.thread
	}
}

impl<T> Debug for ScopedJoinHandle<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.pad("ScopedJoinHandle { .. }")
	}
}

#[expect(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for ScopedJoinHandle<'_, T> {}
unsafe impl<T> Sync for ScopedJoinHandle<'_, T> {}

type SharedVec<T> = Arc<Mutex<Vec<T>>>;
type SharedOption<T> = Arc<Mutex<Option<T>>>;

pub fn scope<'env, R>(f: impl FnOnce(&Scope<'env>) -> R) -> thread::Result<R> {
	struct AbortOnPanic;

	impl Drop for AbortOnPanic {
		fn drop(&mut self) {
			if thread::panicking() {
				std::process::abort();
			}
		}
	}

	let wg = WaitGroup::new();
	let scope = Scope::<'env> {
		handles: SharedVec::default(),
		wait_group: wg.clone(),
		marker: PhantomData,
	};

	let result = panic::catch_unwind(panic::AssertUnwindSafe(|| f(&scope)));

	let guard = AbortOnPanic;

	drop(scope.wait_group);
	wg.wait();

	let panics = scope
		.handles
		.lock()
		.unwrap()
		.drain(..)
		.filter_map(|handle| handle.lock().unwrap().take())
		.filter_map(|handle| handle.join().err())
		.collect::<Vec<_>>();

	mem::forget(guard);

	match result {
		Ok(res) if panics.is_empty() => Ok(res),
		Ok(_) => Err(Box::new(panics)),
		Err(err) => panic::resume_unwind(err),
	}
}
