use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use vmm_core::prettier::{Document, PrettyPrint, const_text, display};

use crate::{LibraryNamespace, LibraryPath, SourceSpan, Spanned, ast::Ident};

#[derive(Clone)]
pub struct Import {
	pub span: SourceSpan,
	pub name: Ident,
	pub path: LibraryPath,
	pub uses: usize,
}

impl Import {
	#[must_use]
	pub fn is_aliased(&self) -> bool {
		self.name != self.path.last()
	}

	#[must_use]
	pub fn namespace(&self) -> &LibraryNamespace {
		self.path.namespace()
	}

	#[must_use]
	pub const fn path(&self) -> &LibraryPath {
		&self.path
	}

	#[must_use]
	pub const fn is_used(&self) -> bool {
		self.uses > 0
	}
}

impl Debug for Import {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Import")
			.field("name", &self.name)
			.field("path", &self.path)
			.field("uses", &self.uses)
			.finish_non_exhaustive()
	}
}

impl Display for Import {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl Eq for Import {}

impl PartialEq for Import {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.path == other.path
	}
}

impl PrettyPrint for Import {
	fn render(&self) -> Document {
		let mut doc = const_text("use") + const_text(".") + display(&self.path);
		if self.is_aliased() {
			doc += const_text("->") + display(&self.name);
		}
		doc
	}
}

impl Spanned for Import {
	fn span(&self) -> SourceSpan {
		self.span
	}
}
