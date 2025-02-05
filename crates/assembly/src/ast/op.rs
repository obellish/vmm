use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use vmm_core::prettier::{Document, PrettyPrint, display, nl, text};

use super::{Block, Instruction};
use crate::{SourceSpan, Span, Spanned};

#[derive(Clone)]
#[repr(u8)]
pub enum Op {
	If {
		span: SourceSpan,
		then_blk: Block,
		else_blk: Block,
	},
	While {
		span: SourceSpan,
		body: Block,
	},
	Repeat {
		span: SourceSpan,
		count: u32,
		body: Block,
	},
	Inst(Span<Instruction>),
}

impl Debug for Op {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::If {
				then_blk, else_blk, ..
			} => f
				.debug_struct("If")
				.field("then", &then_blk)
				.field("else", &else_blk)
				.finish(),
			Self::While { body, .. } => f.debug_tuple("While").field(body).finish(),
			Self::Repeat { count, body, .. } => f
				.debug_struct("Repeat")
				.field("count", &count)
				.field("body", body)
				.finish(),
			Self::Inst(inst) => Debug::fmt(&**inst, f),
		}
	}
}

impl Display for Op {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl Eq for Op {}

impl PartialEq for Op {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(
				Self::If {
					then_blk: lt,
					else_blk: le,
					..
				},
				Self::If {
					then_blk: rt,
					else_blk: re,
					..
				},
			) => lt == rt && le == re,
			(Self::While { body: lbody, .. }, Self::While { body: rbody, .. }) => lbody == rbody,
			(
				Self::Repeat {
					count: lcount,
					body: lbody,
					..
				},
				Self::Repeat {
					count: rcount,
					body: rbody,
					..
				},
			) => lcount == rcount && lbody == rbody,
			(Self::Inst(l), Self::Inst(r)) => l == r,
			_ => false,
		}
	}
}

impl PrettyPrint for Op {
	fn render(&self) -> Document {
		match self {
			Self::If {
				then_blk, else_blk, ..
			} => {
				text("if.true")
					+ nl() + then_blk.render()
					+ nl() + text("else")
					+ nl() + else_blk.render()
					+ nl() + text("end")
			}
			Self::While { body, .. } => {
				text("while.true") + nl() + body.render() + nl() + text("end")
			}
			Self::Repeat { count, body, .. } => {
				display(format_args!("repeat.{count}")) + nl() + body.render() + nl() + text("end")
			}
			Self::Inst(inst) => inst.render(),
		}
	}
}

impl Spanned for Op {
	fn span(&self) -> SourceSpan {
		match self {
			Self::If { span, .. } | Self::While { span, .. } | Self::Repeat { span, .. } => *span,
			Self::Inst(spanned) => spanned.span(),
		}
	}
}
