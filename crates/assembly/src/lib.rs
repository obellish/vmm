#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod ast;
pub mod diagnostics;
mod library;
mod parser;

pub use vmm_core::{mast, utils};

pub use self::{
	diagnostics::{
		DefaultSourceManager, Report, SourceFile, SourceId, SourceManager, SourceSpan, Span,
		Spanned,
	},
	library::{
		LibraryError, LibraryNamespace, LibraryNamespaceError, LibraryPath, LibraryPathComponent,
		PathError, Version, VersionError,
	},
};

const ADVICE_READ_LIMIT: u8 = 16;
const MAX_U32_SHIFT_VALUE: u8 = 31;
const MAX_U32_ROTATE_VALUE: u8 = 31;
const MAX_EXP_BITS: u8 = 64;
