mod context;
mod errors;
mod passes;

use alloc::{
	boxed::Box,
	collections::{BTreeSet, VecDeque},
	sync::Arc,
	vec::Vec,
};

use self::passes::{ConstEvalVisitor, VerifyInvokeTargets};
pub use self::{
	context::AnalysisContext,
	errors::{SemanticAnalysisError, SyntaxError},
};
use crate::{
	LibraryPath, Spanned,
	ast::{
		AliasTarget, Export, Form, Import, Module, ModuleKind, Procedure, ProcedureName,
		Visibility, VisitMut,
	},
	diagnostics::SourceFile,
};

pub fn analyze(
	source: Arc<SourceFile>,
	kind: ModuleKind,
	path: LibraryPath,
	forms: Vec<Form>,
	warnings_as_errors: bool,
) -> Result<Box<Module>, SyntaxError> {
	let mut analyzer = AnalysisContext::new(source.clone());
	analyzer.set_warnings_as_errors(warnings_as_errors);

	let mut module = Box::new(Module::new(kind, path).with_span(source.source_span()));

	let mut forms = VecDeque::from(forms);
	let mut docs = None;
	while let Some(form) = forms.pop_front() {
		match form {
			Form::ModuleDoc(docstring) => {
				assert!(docs.is_none());
				module.set_docs(Some(docstring));
			}
			Form::Doc(docstring) => {
				if let Some(unused) = docs.replace(docstring) {
					analyzer.error(SemanticAnalysisError::UnusedDocstring {
						span: unused.span(),
					});
				}
			}
			Form::Constant(constant) => {
				analyzer.define_constant(constant.with_docs(docs.take()))?;
			}
			Form::Import(import) => {
				if let Some(docs) = docs.take() {
					analyzer.error(SemanticAnalysisError::ImportDocstring { span: docs.span() });
				}
				define_import(import, &mut module, &mut analyzer)?;
			}
			Form::Procedure(export @ Export::Alias(_)) => match kind {
				ModuleKind::Kernel => {
					docs.take();
					analyzer.error(SemanticAnalysisError::ReexportFromKernel {
						span: export.span(),
					});
				}
				ModuleKind::Executable => {
					docs.take();
					analyzer.error(SemanticAnalysisError::UnexpectedExport {
						span: export.span(),
					});
				}
				ModuleKind::Library => {
					define_procedure(export.with_docs(docs.take()), &mut module, &mut analyzer)?;
				}
			},
			Form::Procedure(export) => match kind {
				ModuleKind::Executable
					if export.visibility().is_exported() && !export.is_main() =>
				{
					docs.take();
					analyzer.error(SemanticAnalysisError::UnexpectedExport {
						span: export.span(),
					});
				}
				_ => define_procedure(export.with_docs(docs.take()), &mut module, &mut analyzer)?,
			},
			Form::Begin(body) if matches!(kind, ModuleKind::Executable) => {
				let docs = docs.take();
				let procedure = Procedure::new(
					body.span(),
					Visibility::Public,
					ProcedureName::main(),
					0,
					body,
				)
				.with_docs(docs);
				define_procedure(Export::Procedure(procedure), &mut module, &mut analyzer)?;
			}
			Form::Begin(body) => {
				docs.take();
				analyzer.error(SemanticAnalysisError::UnexpectedEntrypoint { span: body.span() });
			}
		}
	}

	if let Some(unused) = docs.take() {
		analyzer.error(SemanticAnalysisError::UnusedDocstring {
			span: unused.span(),
		});
	}

	if matches!(kind, ModuleKind::Executable) && !module.has_entrypoint() {
		analyzer.error(SemanticAnalysisError::MissingEntryPoint);
	}

	analyzer.has_failed()?;

	visit_procedures(&mut module, &mut analyzer)?;

	for import in module.imports() {
		if !import.is_used() {
			analyzer.error(SemanticAnalysisError::UnusedImport {
				span: import.span(),
			});
		}
	}

	analyzer.into_result().map(move |()| module)
}

fn visit_procedures(
	module: &mut Module,
	analyzer: &mut AnalysisContext,
) -> Result<(), SyntaxError> {
	let is_kernel = module.is_kernel();
	let locals = module
		.procedures()
		.map(|p| p.name().clone())
		.collect::<BTreeSet<_>>();
	let mut procedures = VecDeque::from(core::mem::take(&mut module.procedures));
	while let Some(procedure) = procedures.pop_front() {
		match procedure {
			Export::Procedure(mut procedure) => {
				if is_kernel && matches!(procedure.visibility(), Visibility::Public) {
					procedure.set_visibility(Visibility::SysCall);
				}

				{
					let mut visitor = ConstEvalVisitor::new(analyzer);
					visitor.visit_procedure_mut(&mut procedure);
				}

				{
					let mut visitor = VerifyInvokeTargets::new(
						analyzer,
						module,
						&locals,
						procedure.name().clone(),
					);
					visitor.visit_procedure_mut(&mut procedure);
				}

				module.procedures.push(Export::Procedure(procedure));
			}
			Export::Alias(mut alias) => {
				let is_absolute = alias.is_absolute();
				if !is_absolute {
					if let AliasTarget::ProcedurePath(target) = alias.target_mut() {
						let imported_module =
							target.module.namespace().to_ident().with_span(target.span);
						if let Some(import) = module.resolve_import_mut(&imported_module) {
							target.module = import.path().clone();
							import.uses += 1;
						} else {
							analyzer
								.error(SemanticAnalysisError::MissingImport { span: alias.span() });
						}
					}
				}

				module.procedures.push(Export::Alias(alias));
			}
		}
	}

	Ok(())
}

fn define_import(
	import: Import,
	module: &mut Module,
	context: &mut AnalysisContext,
) -> Result<(), SyntaxError> {
	if let Err(err) = module.define_import(import) {
		match err {
			SemanticAnalysisError::ImportConflict { .. } => {
				context.error(err);
			}
			err => {
				context.error(err);
				context.has_failed()?;
			}
		}
	}

	Ok(())
}

fn define_procedure(
	export: Export,
	module: &mut Module,
	context: &mut AnalysisContext,
) -> Result<(), SyntaxError> {
	let name = export.name().clone();
	if let Err(err) = module.define_procedure(export) {
		match err {
			SemanticAnalysisError::SymbolConflict { .. } => {
				context.error(err);
			}
			err => {
				context.error(err);
				context.has_failed()?;
			}
		}
	}

	context.register_procedure_name(name);

	Ok(())
}
