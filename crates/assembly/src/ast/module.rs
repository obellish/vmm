use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	ops::{Index, IndexMut},
	slice,
};

use vmm_core::prettier::{Document, PrettyPrint, nl, text};

use crate::{
	LibraryNamespace, LibraryPath, SourceSpan, Span, Spanned,
	ast::{
		AliasTarget, Export, Ident, Import, LocalNameResolver, ProcedureIndex, ProcedureName,
		QualifiedProcedureName, ResolvedProcedure,
	},
	diagnostics::{Report, SourceFile},
	sema::SemanticAnalysisError,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Clone)]
pub struct Module {
	span: SourceSpan,
	docs: Option<Span<String>>,
	path: LibraryPath,
	kind: ModuleKind,
	pub(crate) imports: Vec<Import>,
	pub(crate) procedures: Vec<Export>,
}

impl Module {
	pub const FILE_EXTENSION: &'static str = "masm";
	pub const ROOT: &'static str = "mod";
	pub const ROOT_FILENAME: &'static str = "mod.masm";

	#[must_use]
	pub fn new(kind: ModuleKind, path: LibraryPath) -> Self {
		Self {
			span: SourceSpan::default(),
			docs: None,
			path,
			kind,
			imports: Vec::new(),
			procedures: Vec::new(),
		}
	}

	#[must_use]
	pub fn kernel() -> Self {
		Self::new(ModuleKind::Kernel, LibraryNamespace::Kernel.into())
	}

	#[must_use]
	pub fn executable() -> Self {
		Self::new(ModuleKind::Executable, LibraryNamespace::Exec.into())
	}

	#[must_use]
	pub const fn with_span(mut self, span: SourceSpan) -> Self {
		self.span = span;
		self
	}

	pub fn set_path(&mut self, path: LibraryPath) {
		self.path = path;
	}

	pub fn set_namespace(&mut self, ns: LibraryNamespace) {
		self.path.set_namespace(ns);
	}

	pub fn set_docs(&mut self, docs: Option<Span<String>>) {
		self.docs = docs;
	}

	pub fn set_span(&mut self, span: SourceSpan) {
		self.span = span;
	}

	pub fn define_procedure(&mut self, export: Export) -> Result<(), SemanticAnalysisError> {
		if self.is_kernel() && matches!(export, Export::Alias(_)) {
			return Err(SemanticAnalysisError::ReexportFromKernel {
				span: export.span(),
			});
		}
		if let Some(prev) = self.resolve(export.name()) {
			let prev_span = prev.span();
			Err(SemanticAnalysisError::SymbolConflict {
				span: export.span(),
				prev_span,
			})
		} else {
			self.procedures.push(export);
			Ok(())
		}
	}

	pub fn define_import(&mut self, import: Import) -> Result<(), SemanticAnalysisError> {
		if let Some(prev_import) = self.resolve_import(&import.name) {
			let prev_span = prev_import.span;
			return Err(SemanticAnalysisError::ImportConflict {
				span: import.span,
				prev_span,
			});
		}

		if let Some(prev_defined) = self.procedures().find(|e| e.name().eq(&import.name)) {
			let prev_span = prev_defined.span();
			return Err(SemanticAnalysisError::SymbolConflict {
				span: import.span,
				prev_span,
			});
		}

		self.imports.push(import);
		Ok(())
	}

	#[must_use]
	pub fn name(&self) -> &str {
		self.path().last()
	}

	#[must_use]
	pub const fn path(&self) -> &LibraryPath {
		&self.path
	}

	#[must_use]
	pub fn namespace(&self) -> &LibraryNamespace {
		self.path().namespace()
	}

	#[must_use]
	pub fn is_in_namespace(&self, namespace: &LibraryNamespace) -> bool {
		self.namespace() == namespace
	}

	#[must_use]
	pub fn docs(&self) -> Option<Span<&str>> {
		self.docs.as_ref().map(|spanned| spanned.as_deref())
	}

	#[must_use]
	pub const fn kind(&self) -> ModuleKind {
		self.kind
	}

	#[must_use]
	pub const fn is_executable(&self) -> bool {
		self.kind().is_executable()
	}

	#[must_use]
	pub const fn is_kernel(&self) -> bool {
		self.kind().is_kernel()
	}

	pub fn has_entrypoint(&self) -> bool {
		self.index_of(Export::is_main).is_some()
	}

	pub fn procedures(&self) -> slice::Iter<'_, Export> {
		self.procedures.iter()
	}

	pub fn procedures_mut(&mut self) -> slice::IterMut<'_, Export> {
		self.procedures.iter_mut()
	}

	pub fn exported_procedures(
		&self,
	) -> impl Iterator<Item = (ProcedureIndex, QualifiedProcedureName)> + '_ {
		self.procedures().enumerate().filter_map(|(proc_idx, p)| {
			if !p.visibility().is_exported() {
				return None;
			}

			let proc_idx = ProcedureIndex::new(proc_idx);
			let fqn = QualifiedProcedureName::new(self.path().clone(), p.name().clone());

			Some((proc_idx, fqn))
		})
	}

	pub fn imports(&self) -> slice::Iter<'_, Import> {
		self.imports.iter()
	}

	pub fn imports_mut(&mut self) -> slice::IterMut<'_, Import> {
		self.imports.iter_mut()
	}

	pub fn dependencies(&self) -> impl Iterator<Item = &LibraryNamespace> {
		self.import_paths().map(LibraryPath::namespace)
	}

	#[must_use]
	pub fn get(&self, index: ProcedureIndex) -> Option<&Export> {
		self.procedures.get(index.as_usize())
	}

	pub fn index_of(&self, predicate: impl FnMut(&Export) -> bool) -> Option<ProcedureIndex> {
		self.procedures()
			.position(predicate)
			.map(ProcedureIndex::new)
	}

	pub fn import_paths(&self) -> impl Iterator<Item = &LibraryPath> {
		self.imports().map(|import| &import.path)
	}

	#[must_use]
	pub fn index_of_name(&self, name: &ProcedureName) -> Option<ProcedureIndex> {
		self.index_of(|p| p.name() == name && p.visibility().is_exported())
	}

	#[must_use]
	pub fn resolve(&self, name: &ProcedureName) -> Option<ResolvedProcedure> {
		let index = self.index_of(|p| p.name() == name)?;

		match self.get(index)? {
			Export::Procedure(proc) => Some(ResolvedProcedure::Local(Span::new(
				proc.name().span(),
				index,
			))),
			Export::Alias(alias) => match alias.target() {
				AliasTarget::MastRoot(digest) => Some(ResolvedProcedure::MastRoot(**digest)),
				AliasTarget::ProcedurePath(path) | AliasTarget::AbsoluteProcedurePath(path) => {
					Some(ResolvedProcedure::External(path.clone()))
				}
			},
		}
	}

	#[must_use]
	pub fn resolver(&self) -> LocalNameResolver {
		self.procedures()
			.enumerate()
			.map(|(i, p)| match p {
				Export::Procedure(p) => (
					p.name().clone(),
					ResolvedProcedure::Local(Span::new(p.name().span(), ProcedureIndex::new(i))),
				),
				Export::Alias(p) => {
					let target = match p.target() {
						AliasTarget::MastRoot(digest) => ResolvedProcedure::MastRoot(**digest),
						AliasTarget::ProcedurePath(path)
						| AliasTarget::AbsoluteProcedurePath(path) => ResolvedProcedure::External(path.clone()),
					};
					(p.name().clone(), target)
				}
			})
			.collect::<LocalNameResolver>()
			.with_imports(self.imports().map(|import| {
				(
					import.name.clone(),
					Span::new(import.span(), import.path().clone()),
				)
			}))
	}

	#[must_use]
	pub fn resolve_import(&self, module_name: &Ident) -> Option<&Import> {
		self.imports().find(|import| &import.name == module_name)
	}

	pub fn resolve_import_mut(&mut self, module_name: &Ident) -> Option<&mut Import> {
		self.imports_mut()
			.find(|import| &import.name == module_name)
	}
}

impl Debug for Module {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Module")
			.field("docs", &self.docs)
			.field("path", &self.path)
			.field("kind", &self.kind)
			.field("imports", &self.imports)
			.field("procedures", &self.procedures)
			.finish_non_exhaustive()
	}
}

impl Display for Module {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl Eq for Module {}

impl Index<ProcedureIndex> for Module {
	type Output = Export;

	fn index(&self, index: ProcedureIndex) -> &Self::Output {
		&self.procedures[index.as_usize()]
	}
}

impl IndexMut<ProcedureIndex> for Module {
	fn index_mut(&mut self, index: ProcedureIndex) -> &mut Self::Output {
		&mut self.procedures[index.as_usize()]
	}
}

impl PartialEq for Module {
	fn eq(&self, other: &Self) -> bool {
		self.kind() == other.kind()
			&& self.path() == other.path()
			&& self.docs == other.docs
			&& self.imports == other.imports
			&& self.procedures == other.procedures
	}
}

impl PrettyPrint for Module {
	fn render(&self) -> Document {
		let mut doc = Document::Empty;
		if let Some(docs) = self.docs.as_deref() {
			let fragment = docs
				.lines()
				.map(text)
				.reduce(|acc, line| acc + nl() + text("#! ") + line);

			if let Some(fragment) = fragment {
				doc += fragment;
			}
		}

		for (i, import) in self.imports().enumerate() {
			if i > 0 {
				doc += nl();
			}
			doc += import.render();
		}

		if !self.imports.is_empty() {
			doc += nl();
		}

		let mut export_index = 0;
		for export in self.procedures() {
			if export.is_main() {
				continue;
			}
			if export_index > 0 {
				doc += nl();
			}
			doc += export.render();
			export_index += 1;
		}

		if let Some(main) = self.procedures().find(|p| p.is_main()) {
			doc += main.render();
		}

		doc
	}
}

impl Spanned for Module {
	fn span(&self) -> SourceSpan {
		self.span
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModuleKind {
	#[default]
	Library,
	Executable,
	Kernel,
}

impl ModuleKind {
	#[must_use]
	pub const fn is_executable(self) -> bool {
		matches!(self, Self::Executable)
	}

	#[must_use]
	pub const fn is_kernel(self) -> bool {
		matches!(self, Self::Kernel)
	}

	#[must_use]
	pub const fn is_library(self) -> bool {
		matches!(self, Self::Library)
	}
}

impl Deserializable for ModuleKind {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		Ok(match source.read_u8()? {
			0 => Self::Library,
			1 => Self::Executable,
			2 => Self::Kernel,
			n => {
				return Err(DeserializationError::InvalidValue(format!(
					"invalid module tag kind: {n}"
				)));
			}
		})
	}
}

impl Display for ModuleKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Library => "library",
			Self::Executable => "executable",
			Self::Kernel => "kernel",
		})
	}
}

impl Serializable for ModuleKind {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u8(*self as u8);
	}

	fn get_size_hint(&self) -> usize {
		(*self as u8).get_size_hint()
	}
}
