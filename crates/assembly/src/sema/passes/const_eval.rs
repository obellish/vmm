use core::ops::ControlFlow;

use crate::{
	Span, Spanned,
	ast::{ErrorCode, ImmFelt, ImmU8, ImmU16, ImmU32, Immediate, VisitMut},
	sema::{AnalysisContext, SemanticAnalysisError},
};

#[repr(transparent)]
pub struct ConstEvalVisitor<'analyzer> {
	analyzer: &'analyzer mut AnalysisContext,
}

impl<'analyzer> ConstEvalVisitor<'analyzer> {
	pub const fn new(analyzer: &'analyzer mut AnalysisContext) -> Self {
		Self { analyzer }
	}

	fn eval_const<T>(&mut self, imm: &mut Immediate<T>) -> ControlFlow<()>
	where
		T: TryFrom<u64>,
	{
		match imm {
			Immediate::Value(_) => ControlFlow::Continue(()),
			Immediate::Constant(name) => {
				let span = name.span();
				match self.analyzer.get_constant(name) {
					Ok(value) => match T::try_from(value.as_int()) {
						Ok(value) => {
							*imm = Immediate::Value(Span::new(span, value));
						}
						Err(_) => self
							.analyzer
							.error(SemanticAnalysisError::ImmediateOverflow { span }),
					},
					Err(err) => self.analyzer.error(err),
				}
				ControlFlow::Continue(())
			}
		}
	}
}

impl VisitMut for ConstEvalVisitor<'_> {
	fn visit_immediate_u8_mut(&mut self, imm: &mut ImmU8) -> ControlFlow<()> {
		self.eval_const(imm)
	}

	fn visit_immediate_u16_mut(&mut self, imm: &mut ImmU16) -> ControlFlow<()> {
		self.eval_const(imm)
	}

	fn visit_immediate_u32_mut(&mut self, imm: &mut ImmU32) -> ControlFlow<()> {
		self.eval_const(imm)
	}

	fn visit_immediate_error_code_mut(&mut self, code: &mut ErrorCode) -> ControlFlow<()> {
		self.eval_const(code)
	}

	fn visit_immediate_felt_mut(&mut self, imm: &mut ImmFelt) -> ControlFlow<()> {
		match imm {
			Immediate::Value(_) => ControlFlow::Continue(()),
			Immediate::Constant(name) => {
				let span = name.span();
				match self.analyzer.get_constant(name) {
					Ok(value) => {
						*imm = Immediate::Value(Span::new(span, value));
					}
					Err(error) => self.analyzer.error(error),
				}
				ControlFlow::Continue(())
			}
		}
	}
}
