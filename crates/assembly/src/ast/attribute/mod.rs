mod meta;

use vmm_core::prettier;

use crate::{SourceSpan, Spanned, ast::Ident};

pub enum Attribute {
	Marker(Ident),
}
