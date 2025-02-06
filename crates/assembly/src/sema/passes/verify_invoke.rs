use alloc::collections::BTreeSet;
use core::ops::ControlFlow;

use crate::{
	Span, Spanned,
	ast::{
		Ident, Instruction, InvocationTarget, Invoke, InvokeKind, Module, Procedure,
		ProcedureAlias, ProcedureName, VisitMut, visit,
	},
	sema::{AnalysisContext, SemanticAnalysisError},
};

pub struct VerifyInvokeTargets<'a> {
	analyzer: &'a mut AnalysisContext,
	module: &'a mut Module,
	procedures: &'a BTreeSet<ProcedureName>,
	current_procedure: ProcedureName,
	invoked: BTreeSet<Invoke>,
}

impl<'a> VerifyInvokeTargets<'a> {
	pub const fn new(
		analyzer: &'a mut AnalysisContext,
		module: &'a mut Module,
		procedures: &'a BTreeSet<ProcedureName>,
		current_procedure: ProcedureName,
	) -> Self {
		Self {
			analyzer,
			module,
			procedures,
			current_procedure,
			invoked: BTreeSet::new(),
		}
	}

	fn resolve_local(&mut self, name: &ProcedureName) -> ControlFlow<()> {
		if !self.procedures.contains(name) {
			self.analyzer
				.error(SemanticAnalysisError::SymbolUndefined { span: name.span() });
		}

		ControlFlow::Continue(())
	}

	fn resolve_external(
		&mut self,
		name: &ProcedureName,
		module: &Ident,
	) -> Option<InvocationTarget> {
		if let Some(import) = self.module.resolve_import_mut(module) {
			import.uses += 1;
			Some(InvocationTarget::AbsoluteProcedurePath {
				name: name.clone(),
				path: import.path().clone(),
			})
		} else {
			self.analyzer
				.error(SemanticAnalysisError::MissingImport { span: name.span() });
			None
		}
	}
}

impl VisitMut for VerifyInvokeTargets<'_> {
	fn visit_inst_mut(&mut self, inst: &mut Span<Instruction>) -> ControlFlow<()> {
		let span = inst.span();
		match &**inst {
			Instruction::Caller if self.module.is_kernel() => ControlFlow::Continue(()),
			Instruction::Caller => {
				self.analyzer
					.error(SemanticAnalysisError::CallerInKernel { span });
				ControlFlow::Continue(())
			}
			_ => visit::visit_inst_mut(self, inst),
		}
	}

	fn visit_procedure_alias_mut(&mut self, alias: &mut ProcedureAlias) -> ControlFlow<()> {
		if let Some(import) = self.module.resolve_import_mut(alias.name()) {
			import.uses += 1;
		}
		ControlFlow::Continue(())
	}

	fn visit_procedure_mut(&mut self, proc: &mut Procedure) -> ControlFlow<()> {
		let result = visit::visit_procedure_mut(self, proc);
		proc.extend(core::mem::take(&mut self.invoked));
		result
	}

	fn visit_syscall_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
		if self.module.is_kernel() {
			self.analyzer.error(SemanticAnalysisError::SyscallInKernel {
				span: target.span(),
			});
		}

		match target {
			InvocationTarget::ProcedureName(_) => (),
			_ => self.visit_invoke_target_mut(target)?,
		}
		self.invoked
			.insert(Invoke::new(InvokeKind::SysCall, target.clone()));
		ControlFlow::Continue(())
	}

	fn visit_call_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
		if self.module.is_kernel() {
			self.analyzer.error(SemanticAnalysisError::CallInKernel {
				span: target.span(),
			});
		}

		self.visit_invoke_target_mut(target)?;
		self.invoked
			.insert(Invoke::new(InvokeKind::Call, target.clone()));
		ControlFlow::Continue(())
	}

	fn visit_exec_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
		self.visit_invoke_target_mut(target)?;
		self.invoked
			.insert(Invoke::new(InvokeKind::Exec, target.clone()));
		ControlFlow::Continue(())
	}

	fn visit_invoke_target_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
		let span = target.span();
		match target {
			InvocationTarget::MastRoot(_) => {}
			InvocationTarget::AbsoluteProcedurePath { name, path } => {
				if self.module.path() == path && &self.current_procedure == name {
					self.analyzer
						.error(SemanticAnalysisError::SelfRecursive { span });
				}
			}
			InvocationTarget::ProcedureName(name) if name == &self.current_procedure => self
				.analyzer
				.error(SemanticAnalysisError::SelfRecursive { span }),
			InvocationTarget::ProcedureName(name) => return self.resolve_local(name),
			InvocationTarget::ProcedurePath { name, module } => {
				if let Some(new_target) = self.resolve_external(name, module) {
					*target = new_target;
				}
			}
		}

		ControlFlow::Continue(())
	}
}
