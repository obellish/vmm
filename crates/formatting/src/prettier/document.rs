use alloc::{
	borrow::Cow,
	rc::Rc,
	string::{String, ToString},
};
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	mem,
	ops::{Add, AddAssign, BitOr},
};

#[derive(Debug, Default, Clone)]
pub enum Document {
	#[default]
	Empty,
	Newline,
	Char(char, u32),
	Text(Cow<'static, str>, u32),
	Flatten(Rc<Self>),
	Indent(u32, Rc<Self>),
	Concat(Rc<Self>, Rc<Self>),
	Choice(Rc<Self>, Rc<Self>),
}

impl Document {
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		matches!(self, Self::Empty)
	}

	#[must_use]
	pub fn has_leading_newline(&self) -> bool {
		match self {
			Self::Newline | Self::Char('\n' | '\r', _) => true,
			Self::Char(..) | Self::Empty | Self::Choice(..) => false,
			Self::Text(text, _) => text.starts_with(['\n', '\r']),
			Self::Flatten(doc) | Self::Indent(_, doc) => doc.has_leading_newline(),
			Self::Concat(a, b) if a.is_empty() => b.has_leading_newline(),
			Self::Concat(a, _) => a.has_leading_newline(),
		}
	}
}

impl Add for Document {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		if self.is_empty() {
			return rhs;
		}

		if rhs.is_empty() {
			return self;
		}

		Self::Concat(Rc::new(self), Rc::new(rhs))
	}
}

impl Add<&'static str> for Document {
	type Output = Self;

	fn add(self, rhs: &'static str) -> Self::Output {
		let other = const_text(rhs);
		self + other
	}
}

impl Add<Document> for &'static str {
	type Output = Document;

	fn add(self, rhs: Document) -> Self::Output {
		let lhs = const_text(self);
		lhs + rhs
	}
}

impl Add<char> for Document {
	type Output = Self;

	fn add(self, rhs: char) -> Self::Output {
		let other = character(rhs);
		self + other
	}
}

impl Add<Document> for char {
	type Output = Document;

	fn add(self, rhs: Document) -> Self::Output {
		let lhs = character(self);

		lhs + rhs
	}
}

impl AddAssign for Document {
	fn add_assign(&mut self, rhs: Self) {
		if rhs.is_empty() {
			return;
		}

		if self.is_empty() {
			*self = rhs;
			return;
		}

		let lhs = mem::take(self);
		*self = Self::Concat(Rc::new(lhs), Rc::new(rhs));
	}
}

impl AddAssign<&'static str> for Document {
	fn add_assign(&mut self, rhs: &'static str) {
		let rhs = const_text(rhs);

		self.add_assign(rhs);
	}
}

impl AddAssign<char> for Document {
	fn add_assign(&mut self, rhs: char) {
		let rhs = character(rhs);

		self.add_assign(rhs);
	}
}

impl BitOr for Document {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		if self.is_empty() {
			return rhs;
		}

		if rhs.is_empty() {
			return self;
		}

		Self::Choice(Rc::new(self), Rc::new(rhs))
	}
}

impl Display for Document {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Empty => Ok(()),
			Self::Newline => f.write_char('\n'),
			Self::Char(c, _) => f.write_char(*c),
			doc => {
				let width = f.width().unwrap_or(80);
				super::print::pretty_print(doc, width, f)
			}
		}
	}
}

impl From<&'static str> for Document {
	fn from(value: &'static str) -> Self {
		const_text(value)
	}
}

impl From<char> for Document {
	fn from(value: char) -> Self {
		character(value)
	}
}

impl From<String> for Document {
	fn from(value: String) -> Self {
		text(value)
	}
}

#[must_use]
pub const fn nl() -> Document {
	Document::Newline
}

pub fn display(s: impl Display) -> Document {
	let string = Cow::<'_, str>::Owned(s.to_string());
	text(string)
}

pub fn character(c: char) -> Document {
	match c {
		'\n' => nl(),
		c => {
			let width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0) as u32;
			Document::Char(c, width)
		}
	}
}

pub fn text(s: impl ToString) -> Document {
	let string = Cow::<'_, str>::Owned(s.to_string());
	let mut chars = string.chars();
	match chars.next() {
		None => Document::Empty,
		Some(c) if chars.next().is_none() => character(c),
		Some(_) => {
			drop(chars);
			let width = unicode_width::UnicodeWidthStr::width(string.as_ref()) as u32;
			Document::Text(string, width)
		}
	}
}

#[must_use]
pub fn const_text(s: &'static str) -> Document {
	let mut chars = s.chars();
	match chars.next() {
		None => Document::Empty,
		Some(c) if chars.next().is_none() => character(c),
		Some(_) => {
			drop(chars);
			let string = Cow::Borrowed(s);
			let width = unicode_width::UnicodeWidthStr::width(string.as_ref()) as u32;
			Document::Text(string, width)
		}
	}
}

pub fn split(input: impl AsRef<str>) -> Document {
	let input = input.as_ref();
	input
		.lines()
		.map(text)
		.reduce(|acc, doc| match acc {
			Document::Empty => doc + nl(),
			other => other + doc + nl(),
		})
		.unwrap_or_default()
}

#[must_use]
pub fn concat(left: Document, right: Document) -> Document {
	left + right
}

#[must_use]
pub fn flatten(doc: Document) -> Document {
	if doc.is_empty() {
		doc
	} else {
		Document::Flatten(Rc::new(doc))
	}
}

#[must_use]
pub fn indent(indent: u32, doc: Document) -> Document {
	if doc.is_empty() {
		doc
	} else {
		Document::Indent(indent, Rc::new(doc))
	}
}
