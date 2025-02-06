use alloc::{boxed::Box, string::String};
use core::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _};

use vmm_core::{
	Felt, FieldElement,
	prettier::{Document, PrettyPrint, const_text, display, flatten, nl, text},
};

use crate::{SourceSpan, Span, Spanned, ast::Ident, parser::ParsingError};

pub struct Constant {
	pub span: SourceSpan,
	pub docs: Option<Span<String>>,
	pub name: Ident,
	pub value: ConstantExpr,
}

impl Constant {
	#[must_use]
	pub const fn new(span: SourceSpan, name: Ident, value: ConstantExpr) -> Self {
		Self {
			span,
			docs: None,
			name,
			value,
		}
	}

	#[must_use]
	pub fn with_docs(mut self, docs: Option<Span<String>>) -> Self {
		self.docs = docs;
		self
	}
}

impl Debug for Constant {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Constant")
			.field("docs", &self.docs)
			.field("name", &self.name)
			.field("value", &self.value)
			.finish_non_exhaustive()
	}
}

impl Eq for Constant {}

impl PartialEq for Constant {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.value == other.value
	}
}

impl PrettyPrint for Constant {
	fn render(&self) -> Document {
		let mut doc = Document::Empty;
		if let Some(docs) = self.docs.as_deref() {
			let fragment = docs
				.lines()
				.map(text)
				.reduce(|acc, line| acc + nl() + const_text("#! ") + line);

			if let Some(fragment) = fragment {
				doc += fragment;
			}
		}

		doc += nl();
		doc += flatten(const_text("const") + const_text(".") + display(&self.name));
		doc += const_text("=");

		doc + self.value.render()
	}
}

impl Spanned for Constant {
	fn span(&self) -> SourceSpan {
		self.span
	}
}

pub enum ConstantExpr {
	Literal(Span<Felt>),
	Var(Ident),
	BinaryOp {
		span: SourceSpan,
		op: ConstantOp,
		lhs: Box<Self>,
		rhs: Box<Self>,
	},
}

impl ConstantExpr {
	#[must_use]
	#[track_caller]
	pub fn unwrap_literal(&self) -> Felt {
		match self {
			Self::Literal(spanned) => spanned.into_inner(),
			other => panic!("expected constant expression to be a literal, got {other:#?}"),
		}
	}

	pub fn try_fold(self) -> Result<Self, ParsingError> {
		match self {
			Self::Literal(_) | Self::Var(_) => Ok(self),
			Self::BinaryOp { span, op, lhs, rhs } => {
				if rhs.is_literal() {
					let rhs = Self::into_inner(rhs).try_fold()?;
					match rhs {
						Self::Literal(rhs) => {
							let lhs = Self::into_inner(lhs).try_fold()?;
							match lhs {
								Self::Literal(lhs) => {
									let lhs = lhs.into_inner();
									let rhs = rhs.into_inner();
									let is_division =
										matches!(op, ConstantOp::Div | ConstantOp::IntDiv);
									let is_division_by_zero = is_division && rhs == Felt::ZERO;
									if is_division_by_zero {
										return Err(ParsingError::DivisionByZero { span });
									}
									match op {
										ConstantOp::Add => {
											Ok(Self::Literal(Span::new(span, lhs + rhs)))
										}
										ConstantOp::Sub => {
											Ok(Self::Literal(Span::new(span, lhs - rhs)))
										}
										ConstantOp::Mul => {
											Ok(Self::Literal(Span::new(span, lhs * rhs)))
										}
										ConstantOp::Div => {
											Ok(Self::Literal(Span::new(span, lhs / rhs)))
										}
										ConstantOp::IntDiv => Ok(Self::Literal(Span::new(
											span,
											Felt::new(lhs.as_int() / rhs.as_int()),
										))),
									}
								}
								lhs => Ok(Self::BinaryOp {
									span,
									op,
									lhs: Box::new(lhs),
									rhs: Box::new(Self::Literal(rhs)),
								}),
							}
						}
						rhs => {
							let lhs = Self::into_inner(lhs).try_fold()?;
							Ok(Self::BinaryOp {
								span,
								op,
								lhs: Box::new(lhs),
								rhs: Box::new(rhs),
							})
						}
					}
				} else {
					let lhs = Self::into_inner(lhs).try_fold()?;
					Ok(Self::BinaryOp {
						span,
						op,
						lhs: Box::new(lhs),
						rhs,
					})
				}
			}
		}
	}

	fn is_literal(&self) -> bool {
		match self {
			Self::Literal(_) => true,
			Self::Var(_) => false,
			Self::BinaryOp { lhs, rhs, .. } => lhs.is_literal() && rhs.is_literal(),
		}
	}

	#[allow(clippy::boxed_local)]
	fn into_inner(self: Box<Self>) -> Self {
		*self
	}
}

impl Debug for ConstantExpr {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Literal(lit) => Debug::fmt(&**lit, f),
			Self::Var(name) => Debug::fmt(&**name, f),
			Self::BinaryOp { op, lhs, rhs, .. } => {
				f.debug_tuple(op.name()).field(lhs).field(rhs).finish()
			}
		}
	}
}

impl Display for ConstantExpr {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl Eq for ConstantExpr {}

impl PartialEq for ConstantExpr {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Literal(l), Self::Literal(r)) => l == r,
			(Self::Var(l), Self::Var(r)) => l == r,
			(
				Self::BinaryOp {
					op: lop,
					lhs: llhs,
					rhs: lrhs,
					..
				},
				Self::BinaryOp {
					op: rop,
					lhs: rlhs,
					rhs: rrhs,
					..
				},
			) => lop == rop && llhs == rlhs && lrhs == rrhs,
			_ => false,
		}
	}
}

impl PrettyPrint for ConstantExpr {
	fn render(&self) -> Document {
		match self {
			Self::Literal(literal) => display(literal),
			Self::Var(ident) => display(ident),
			Self::BinaryOp { op, lhs, rhs, .. } => {
				let single_line = lhs.render() + display(op) + rhs.render();
				let multi_line = lhs.render() + nl() + (display(op)) + rhs.render();
				single_line | multi_line
			}
		}
	}
}

impl Spanned for ConstantExpr {
	fn span(&self) -> SourceSpan {
		match self {
			Self::Literal(spanned) => spanned.span(),
			Self::Var(spanned) => spanned.span(),
			Self::BinaryOp { span, .. } => *span,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantOp {
	Add,
	Sub,
	Mul,
	Div,
	IntDiv,
}

impl ConstantOp {
	const fn name(self) -> &'static str {
		match self {
			Self::Add => "Add",
			Self::Sub => "Sub",
			Self::Mul => "Mul",
			Self::Div => "Div",
			Self::IntDiv => "IntDiv",
		}
	}
}

impl Display for ConstantOp {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Add => f.write_char('+'),
			Self::Sub => f.write_char('-'),
			Self::Mul => f.write_char('*'),
			Self::Div => f.write_char('/'),
			Self::IntDiv => f.write_str("//"),
		}
	}
}
