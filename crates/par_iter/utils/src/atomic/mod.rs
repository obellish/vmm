#[cfg(target_has_atomic = "ptr")]
mod cell;
mod consume;
#[cfg(target_has_atomic = "ptr")]
#[cfg_attr(
	any(target_pointer_width = "16", target_pointer_width = "32"),
	path = "seq_lock_wide.rs"
)]
mod seq_lock;

#[cfg(target_has_atomic = "ptr")]
pub use self::cell::AtomicCell;
pub use self::consume::AtomicConsume;
