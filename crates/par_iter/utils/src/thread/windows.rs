use std::os::windows::io::{AsRawHandle, IntoRawHandle, RawHandle};

use super::ScopedJoinHandle;

impl<T> AsRawHandle for ScopedJoinHandle<'_, T> {
	fn as_raw_handle(&self) -> RawHandle {
		let handle = self.handle.lock().unwrap();
		handle.as_ref().unwrap().as_raw_handle()
	}
}

impl<T> IntoRawHandle for ScopedJoinHandle<'_, T> {
	fn into_raw_handle(self) -> RawHandle {
		self.as_raw_handle()
	}
}
