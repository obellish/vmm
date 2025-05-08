mod noop;

pub use self::noop::*;

use color_eyre::Result;
use serde::Serialize;

/// Intermediate Optimizations store (aka results of things between passes).
pub trait IROptStore {
    /// Write a raw, serializable value to the store
    fn write_value<S: Serialize>(&self, iteration: usize, value: &S) -> Result<()>;
}
