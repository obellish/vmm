use std::sync::atomic::{AtomicBool, Ordering};

use color_eyre::eyre::Result;
use serde::Serialize;
use tracing::warn;

use super::{OptStore, OptStoreError};

static WROTE_WARNING: AtomicBool = AtomicBool::new(false);

pub struct NoOpStore;

impl OptStore for NoOpStore {
	fn write_value<S: Serialize>(&self, iteration: usize, value: &S) -> Result<(), OptStoreError> {
		if !WROTE_WARNING.load(Ordering::SeqCst) {
			warn!("NoOpStore chosen as output, not exposing any intermediate steps");
			WROTE_WARNING.store(true, Ordering::SeqCst);
		}

		Ok(())
	}

	fn read_value<'de, S>(&self, iteration: usize) -> Option<S>
	where
		S: serde::Deserialize<'de>,
	{
		if !WROTE_WARNING.load(Ordering::SeqCst) {
			warn!("NoOpStore chosen as output, not exposing any intermediate steps");
			WROTE_WARNING.store(true, Ordering::SeqCst);
		}

		None
	}
}
