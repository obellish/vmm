use thiserror::Error;
use vmm_core::errors::KernelError;

use crate::{ast::QualifiedProcedureName, diagnostics::Diagnostic};

#[derive(Debug, Error, Diagnostic)]
pub enum LibraryError {
	#[error("library must contain at least one exported procedure")]
	#[diagnostic()]
	NoExport,
	#[error("invalid export in kernel library: {procedure_path}")]
	InvalidKernelExport {
		procedure_path: QualifiedProcedureName,
	},
	#[error("failed to convert library into kernel library: {0}")]
	KernelConversion(#[from] KernelError),
	#[error("invalid export: no procedure root for {procedure_path} procedure")]
	NoProcedureRootForExport {
		procedure_path: QualifiedProcedureName,
	},
}
