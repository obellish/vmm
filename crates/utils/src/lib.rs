#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "copy_writer")]
mod copy_writer;
#[cfg(feature = "get_or_zero")]
mod get_or_zero;
#[cfg(feature = "heap_size")]
mod heap_size;
#[cfg(feature = "insert_or_push")]
mod insert_or_push;

#[cfg(feature = "copy_writer")]
pub use self::copy_writer::CopyWriter;
#[cfg(feature = "get_or_zero")]
pub use self::get_or_zero::GetOrZero;
#[cfg(feature = "heap_size")]
pub use self::heap_size::HeapSize;
#[cfg(feature = "insert_or_push")]
pub use self::insert_or_push::InsertOrPush;
