use alloc::{
	collections::{BTreeMap, BTreeSet},
	sync::Arc,
	vec::Vec,
};
use core::mem;

use vmm_core::Felt;

use super::{SemanticAnalysisError, SyntaxError};
use crate::{
	SourceFile, Span, Spanned,
	ast::{Constant, ConstantExpr, ConstantOp, Ident, ProcedureName},
	diagnostics::{Diagnostic, Severity},
};

pub struct AnalysisContext {
	constants: BTreeMap<Ident, Constant>,
	procedures: BTreeSet<ProcedureName>,
	errors: Vec<SemanticAnalysisError>,
	source_file: Arc<SourceFile>,
	warnings_as_errors: bool,
}

impl AnalysisContext {
	pub const fn new(source_file: Arc<SourceFile>) -> Self {
		Self {
			constants: BTreeMap::new(),
			procedures: BTreeSet::new(),
			errors: Vec::new(),
			source_file,
			warnings_as_errors: false,
		}
	}

	pub const fn warnings_as_errors(&self) -> bool {
		self.warnings_as_errors
	}

	pub fn warnings_as_errors_mut(&mut self) -> &mut bool {
		&mut self.warnings_as_errors
	}

	pub fn set_warnings_as_errors(&mut self, yes: bool) {
		*self.warnings_as_errors_mut() = yes;
	}

	pub fn register_procedure_name(&mut self, name: ProcedureName) {
		self.procedures.insert(name);
	}

	pub fn define_constant(&mut self, mut constant: Constant) -> Result<(), SyntaxError> {
		if let Some(value) = self.constants.get(&constant.name) {
			self.error(SemanticAnalysisError::SymbolConflict {
				span: constant.span(),
				prev_span: value.span(),
			});
			return Ok(());
		}

		match self.const_eval(&constant.value) {
			Ok(value) => {
				constant.value = ConstantExpr::Literal(Span::new(constant.span(), value));
				self.constants.insert(constant.name.clone(), constant);
				Ok(())
			}
			Err(err) => {
				self.error(err);
				let errors = mem::take(&mut self.errors);
				Err(SyntaxError {
					source_file: self.source_file.clone(),
					errors,
				})
			}
		}
	}

	fn const_eval(&self, value: &ConstantExpr) -> Result<Felt, SemanticAnalysisError> {
		match value {
			ConstantExpr::Literal(value) => Ok(value.into_inner()),
			ConstantExpr::Var(name) => self.get_constant(name),
			ConstantExpr::BinaryOp { op, lhs, rhs, .. } => {
				let rhs = self.const_eval(rhs)?;
				let lhs = self.const_eval(lhs)?;
				match op {
					ConstantOp::Add => Ok(lhs + rhs),
					ConstantOp::Sub => Ok(lhs - rhs),
					ConstantOp::Mul => Ok(lhs * rhs),
					ConstantOp::Div => Ok(lhs / rhs),
					ConstantOp::IntDiv => Ok(Felt::new(lhs.as_int() / rhs.as_int())),
				}
			}
		}
	}

	pub fn get_constant(&self, name: &Ident) -> Result<Felt, SemanticAnalysisError> {
		let span = name.span();
		self.constants.get(name).map_or(
			Err(SemanticAnalysisError::SymbolUndefined { span }),
			|expr| Ok(expr.value.unwrap_literal()),
		)
	}

	pub fn error(&mut self, diagnostic: SemanticAnalysisError) {
		self.errors.push(diagnostic);
	}

	pub fn has_errors(&self) -> bool {
		if self.warnings_as_errors() {
			return !self.errors.is_empty();
		}

		self.errors
			.iter()
			.any(|err| matches!(err.severity().unwrap_or(Severity::Error), Severity::Error))
	}

	pub fn has_failed(&mut self) -> Result<(), SyntaxError> {
		if self.has_errors() {
			Err(SyntaxError {
				source_file: self.source_file.clone(),
				errors: mem::take(&mut self.errors),
			})
		} else {
			Ok(())
		}
	}

	pub fn into_result(self) -> Result<(), SyntaxError> {
		if self.has_errors() {
			Err(SyntaxError {
				source_file: self.source_file.clone(),
				errors: self.errors,
			})
		} else {
			self.emit_warnings();
			Ok(())
		}
	}

	#[cfg(feature = "std")]
	fn emit_warnings(self) {
		use crate::diagnostics::Report;

		if !self.errors.is_empty() {
			let warning = Report::from(super::errors::SyntaxWarning {
				source_file: self.source_file,
				errors: self.errors,
			});

			std::eprintln!("{warning}");
		}
	}

	#[cfg(not(feature = "std"))]
	fn emit_warnings(self) {}
}
