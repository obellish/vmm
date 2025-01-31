pub mod racy_lock;
pub mod rw_lock;

#[cfg(feature = "std")]
pub use std::sync::LazyLock;

#[cfg(feature = "std")]
pub use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(not(feature = "std"))]
pub use self::{
	racy_lock::RacyLock as LazyLock,
	rw_lock::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
