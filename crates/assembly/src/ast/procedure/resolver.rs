use alloc::{collections::BTreeMap, vec::Vec};

use vmm_core::crypto::hash::RpoDigest;

use super::{ProcedureIndex, ProcedureName, QualifiedProcedureName};
use crate::{LibraryPath, SourceSpan, Span, Spanned, ast::Ident};

pub struct LocalNameResolver {
	imports: BTreeMap<Ident, Span<LibraryPath>>,
	resolved: BTreeMap<ProcedureName, ProcedureIndex>,
	resolutions: Vec<ResolvedProcedure>,
}

impl LocalNameResolver {
	#[must_use]
	pub fn resolve(&self, name: &ProcedureName) -> Option<ResolvedProcedure> {
		self.resolved
			.get(name)
			.copied()
			.map(|index| self.resolutions[index.as_usize()].clone())
	}

	#[must_use]
	pub fn resolve_import(&self, name: &Ident) -> Option<Span<&LibraryPath>> {
		self.imports.get(name).map(|spanned| spanned.as_ref())
	}

	#[must_use]
	pub fn get_name(&self, index: ProcedureIndex) -> &ProcedureName {
		self.resolved
			.iter()
			.find_map(|(k, v)| (v == &index).then_some(k))
			.expect("invalid procedure index")
	}

	#[must_use]
	pub fn with_imports(
		mut self,
		imports: impl IntoIterator<Item = (Ident, Span<LibraryPath>)>,
	) -> Self {
		self.imports.extend(imports);
		self
	}
}

impl FromIterator<(ProcedureName, ResolvedProcedure)> for LocalNameResolver {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = (ProcedureName, ResolvedProcedure)>,
	{
		let mut resolver = Self {
			imports: BTreeMap::new(),
			resolved: BTreeMap::new(),
			resolutions: Vec::new(),
		};

		for (name, resolution) in iter {
			let index = ProcedureIndex::new(resolver.resolutions.len());
			resolver.resolutions.push(resolution);
			resolver.resolved.insert(name, index);
		}

		resolver
	}
}

#[derive(Debug, Clone)]
pub enum ResolvedProcedure {
	Local(Span<ProcedureIndex>),
	External(QualifiedProcedureName),
	MastRoot(RpoDigest),
}

impl Spanned for ResolvedProcedure {
	fn span(&self) -> SourceSpan {
		match self {
			Self::Local(p) => p.span(),
			Self::External(p) => p.span(),
			Self::MastRoot(_) => SourceSpan::default(),
		}
	}
}
