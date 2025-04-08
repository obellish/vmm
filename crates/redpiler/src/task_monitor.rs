use std::sync::{
	Arc,
	atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
};

use parking_lot::Mutex;

#[derive(Default)]
pub struct TaskMonitor {
	cancelled: AtomicBool,
	max_progress: AtomicUsize,
	progress: AtomicUsize,
	message: Mutex<Option<Arc<String>>>,
}

impl TaskMonitor {
	pub fn cancel(&self) {
		self.cancelled.store(true, Relaxed);
	}

	pub fn cancelled(&self) -> bool {
		self.cancelled.load(Relaxed)
	}

	pub fn set_progress(&self, progress: usize) {
		self.progress.store(progress, Relaxed);
	}

	pub fn increment_progress(&self) {
		self.progress.fetch_add(1, Relaxed);
	}

	pub fn set_max_progress(&self, max_progress: usize) {
		self.max_progress.store(max_progress, Relaxed);
	}

	pub fn progress(&self) -> usize {
		self.progress.load(Relaxed)
	}

	pub fn max_progress(&self) -> usize {
		self.max_progress.load(Relaxed)
	}

	pub fn message(&self) -> Option<Arc<String>> {
		self.message.lock().clone()
	}

	pub fn set_message(&self, message: impl Into<String>) {
		*self.message.lock() = Some(Arc::new(message.into()));
	}
}
