#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod iter;
mod math;

pub use rayon_core::{
	BroadcastContext, FnContext, Scope, ThreadBuilder, ThreadPool, ThreadPoolBuildError,
	ThreadPoolBuilder, Yield, broadcast, current_num_threads, current_thread_index, in_place_scope,
	in_place_scope_fifo, join, join_context, max_num_threads, scope, scope_fifo, spawn,
	spawn_broadcast, spawn_fifo, yield_local, yield_now,
};

#[repr(transparent)]
struct SendPtr<T: ?Sized>(*mut T);

impl<T: ?Sized> SendPtr<T> {
	const fn get(self) -> *mut T {
		self.0
	}
}

impl<T: ?Sized> Clone for SendPtr<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for SendPtr<T> {}

unsafe impl<T> Send for SendPtr<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for SendPtr<T> where T: ?Sized + Send {}
