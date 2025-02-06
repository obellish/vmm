use alloc::{borrow::Cow, boxed::Box, string::String, sync::Arc, vec::Vec};
use core::fmt::{Formatter, Result as FmtResult};

use regex::Regex;
use vmm_core::Program;

#[cfg(feature = "std")]
use crate::diagnostics::reporting::set_panic_hook;

pub enum Pattern {
	Literal(Cow<'static, str>),
	Regex(Regex),
}
