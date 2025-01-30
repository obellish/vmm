use alloc::vec::Vec;
use core::fmt::{Formatter, Result as FmtResult, Write as _};

use super::Document;

struct PrettyPrinter<'a> {
	width: usize,
	column: u32,
	chunks: Vec<Chunk<'a>>,
}

impl<'a> PrettyPrinter<'a> {
	fn new(doc: &'a Document, width: usize) -> Self {
		let chunk = Chunk {
			doc,
			indent: 0,
			flat: false,
		};
		Self {
			width,
			column: 0,
			chunks: vec![chunk],
		}
	}

	fn print(&mut self, f: &mut Formatter<'_>) -> FmtResult {
		while let Some(chunk) = self.chunks.pop() {
			match chunk.doc {
				Document::Empty => (),
				Document::Newline | Document::Char('\n', _) => {
					f.write_char('\n')?;

					let strip_indentation = self
						.chunks
						.iter()
						.rev()
						.find(|chunk| !chunk.doc.is_empty())
						.is_none_or(|chunk| chunk.doc.has_leading_newline());

					if strip_indentation {
						self.column = 0;
					} else {
						write!(f, "{1:0$}", chunk.indent as usize, "")?;
						self.column = chunk.indent;
					}
				}
				Document::Char(c, width) => {
					f.write_char(*c)?;
					self.column += width;
				}
				Document::Text(text, width) => {
					f.write_str(text)?;
					self.column += width;
				}
				Document::Flatten(x) => self.chunks.push(chunk.flat(x)),
				Document::Indent(i, x) => self.chunks.push(chunk.indented(*i, x)),
				Document::Concat(x, y) => {
					self.chunks.extend([chunk.with_doc(y), chunk.with_doc(x)]);
				}
				Document::Choice(x, y) => {
					if chunk.flat || self.fits(chunk.with_doc(x)) {
						self.chunks.push(chunk.with_doc(x));
					} else {
						self.chunks.push(chunk.with_doc(y));
					}
				}
			}
		}

		Ok(())
	}

	fn fits(&self, chunk: Chunk<'a>) -> bool {
		let mut remaining = self.width.saturating_sub(self.column as usize);
		let mut stack = vec![chunk];
		let mut chunks = self.chunks.as_slice();

		loop {
			let chunk = match stack.pop() {
				Some(chunk) => chunk,
				None => match chunks.split_last() {
					None => return true,
					Some((chunk, more_chunks)) => {
						chunks = more_chunks;
						*chunk
					}
				},
			};

			match &chunk.doc {
				Document::Empty | Document::Newline => return true,
				Document::Char(_, text_width) | Document::Text(_, text_width) => {
					if *text_width as usize <= remaining {
						remaining -= *text_width as usize;
					} else {
						return false;
					}
				}
				Document::Flatten(x) => stack.push(chunk.flat(x)),
				Document::Indent(i, x) => stack.push(chunk.indented(*i, x)),
				Document::Concat(x, y) => {
					stack.extend([chunk.with_doc(y), chunk.with_doc(x)]);
				}
				Document::Choice(x, y) => {
					stack.push(chunk.with_doc(if chunk.flat { x } else { y }));
				}
			}
		}
	}
}

#[derive(Debug, Clone, Copy)]
struct Chunk<'a> {
	doc: &'a Document,
	indent: u32,
	flat: bool,
}

impl<'a> Chunk<'a> {
	const fn with_doc(self, doc: &'a Document) -> Self {
		Self {
			doc,
			indent: self.indent,
			flat: self.flat,
		}
	}

	const fn indented(self, indent: u32, doc: &'a Document) -> Self {
		Self {
			doc,
			indent: self.indent + indent,
			flat: self.flat,
		}
	}

	const fn flat(self, doc: &'a Document) -> Self {
		Self {
			doc,
			indent: self.indent,
			flat: true,
		}
	}
}

pub fn pretty_print(doc: &Document, width: usize, f: &mut Formatter<'_>) -> FmtResult {
	let mut printer = PrettyPrinter::new(doc, width);

	printer.print(f)
}
