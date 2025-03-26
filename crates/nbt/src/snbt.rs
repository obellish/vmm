use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	iter::Peekable,
	str::Chars,
};

use super::{Compound, List, Value};

const STRING_MAX_LENGTH: usize = 32767;
const MAX_DEPTH: usize = 512;

#[derive(Debug)]
pub struct SnbtReader<'a> {
	line: usize,
	column: usize,
	index: usize,
	depth: usize,
	iter: Peekable<Chars<'a>>,
	pushed_back: Option<char>,
}

impl<'a> SnbtReader<'a> {
	#[must_use]
	pub fn new(input: &'a str) -> Self {
		Self {
			line: 1,
			column: 1,
			index: 0,
			depth: 0,
			iter: input.chars().peekable(),
			pushed_back: None,
		}
	}

	fn check_depth<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
		if self.depth >= MAX_DEPTH {
			Err(self.make_error(SnbtErrorType::DepthLimitExceeded))
		} else {
			self.depth += 1;
			let res = f(self);
			self.depth -= 1;
			res
		}
	}

	const fn make_error(&self, kind: SnbtErrorType) -> SnbtError {
		SnbtError::new(kind, self.line, self.column)
	}

	const fn fault(&self, kind: SnbtErrorType) -> Result<()> {
		Err(self.make_error(kind))
	}

	fn peek(&mut self) -> Result<char> {
		if let Some(c) = self.pushed_back {
			Ok(c)
		} else {
			self.iter
				.peek()
				.copied()
				.ok_or_else(|| self.make_error(SnbtErrorType::ReachEndOfStream))
		}
	}

	fn next(&mut self) {
		if self.pushed_back.is_some() {
			self.pushed_back = None;
			return;
		}

		let result = self.iter.next();

		if let Some(c) = result {
			if matches!(c, '\n') {
				self.line += 1;
				self.column = 1;
			} else {
				self.column += 1;
			}

			self.index += c.len_utf8();
		}
	}

	fn push_back(&mut self, c: char) {
		if matches!(c, '\n') {
			self.line -= 1;
			self.column = 1;
		} else {
			self.column -= 1;
		}

		self.index -= c.len_utf8();

		match self.pushed_back {
			Some(_) => panic!("can't push back two chars"),
			None => self.pushed_back = Some(c),
		}
	}

	fn skip_whitespace(&mut self) {
		loop {
			match self.peek() {
				Ok(c) if c.is_whitespace() => self.next(),
				_ => break,
			}
		}
	}

	fn read_string(&mut self) -> Result<String> {
		let first = self.peek()?;

		let str = match first {
			'\"' | '\'' => self.read_quoted_string(),
			_ => self.read_unquoted_string(),
		}?;

		if str.len() > STRING_MAX_LENGTH {
			self.fault(SnbtErrorType::LongString)?;
		}

		Ok(str)
	}

	fn read_unquoted_string(&mut self) -> Result<String> {
		let mut result = String::new();

		loop {
			let input = self.peek();
			match input {
				Ok('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '+' | '.') => {
					result.push(input?);
					self.next();
				}
				_ => break Ok(result),
			}
		}
	}

	fn read_quoted_string(&mut self) -> Result<String> {
		let quote = self.peek()?;
		self.next();

		let mut result = String::new();

		loop {
			let input = self.peek();
			match input {
				Ok(c) if c == quote => {
					self.next();
					break;
				}
				Ok('\\') => {
					self.next();

					let escape = self.peek()?;
					if escape == quote || matches!(escape, '\\') {
						result.push(escape);
					} else {
						self.fault(SnbtErrorType::InvalidEscapeSequence)?;
					}

					self.next();
				}
				Ok(c) => {
					result.push(c);
					self.next();
				}
				Err(e) => return Err(e),
			}
		}

		if result.len() > STRING_MAX_LENGTH {
			self.fault(SnbtErrorType::LongString)?;
		}

		Ok(result)
	}

	fn parse_compound(&mut self) -> Result<Compound> {
		self.next();
		self.skip_whitespace();

		let mut cpd = Compound::new();

		while !matches!(self.peek()?, '}') {
			let key = self.read_string()?;

			self.skip_whitespace();

			if key.is_empty() {
				self.fault(SnbtErrorType::EmptyKeyInCompound)?;
			}

			if !matches!(self.peek()?, ':') {
				self.fault(SnbtErrorType::ExpectColon)?;
			}

			self.next();
			self.skip_whitespace();

			let value = self.parse_element()?;

			self.skip_whitespace();
			if matches!(self.peek()?, ',') {
				self.next();
				self.skip_whitespace();
			} else if !matches!(self.peek()?, '}') {
				self.fault(SnbtErrorType::ExpectComma)?;
			}

			cpd.insert(key, value);
		}

		self.next();
		Ok(cpd)
	}

	fn continue_parse_list(&mut self) -> Result<List> {
		self.skip_whitespace();

		let mut list = List::End;

		while !matches!(self.peek()?, ']') {
			let value = self.parse_element()?;
			self.skip_whitespace();

			match (&mut list, value) {
				(list @ List::End, value) => *list = value.into(),
				(List::Byte(l), Value::Byte(v)) => l.push(v),
				(List::Short(l), Value::Short(v)) => l.push(v),
				(List::Int(l), Value::Int(v)) => l.push(v),
				(List::Long(l), Value::Long(v)) => l.push(v),
				(List::Float(l), Value::Float(v)) => l.push(v),
				(List::Double(l), Value::Double(v)) => l.push(v),
				(List::ByteArray(l), Value::ByteArray(v)) => l.push(v),
				(List::String(l), Value::String(v)) => l.push(v),
				(List::List(l), Value::List(v)) => l.push(v),
				(List::Compound(l), Value::Compound(v)) => l.push(v),
				(List::IntArray(l), Value::IntArray(v)) => l.push(v),
				(List::LongArray(l), Value::LongArray(v)) => l.push(v),
				_ => {
					self.fault(SnbtErrorType::DifferentTypesInList)?;
				}
			}

			if matches!(self.peek()?, ',') {
				self.next();
				self.skip_whitespace();
			} else if !matches!(self.peek()?, ']') {
				self.fault(SnbtErrorType::ExpectComma)?;
			}
		}

		self.next();
		Ok(list)
	}

	fn parse_list_like(&mut self) -> Result<Value> {
		self.next();

		let type_char = self.peek()?;

		let mut values = match type_char {
			'B' => Value::ByteArray(Vec::new()),
			'I' => Value::IntArray(Vec::new()),
			'L' => Value::LongArray(Vec::new()),
			_ => return self.check_depth(|v| Ok(v.continue_parse_list()?.into())),
		};

		self.next();

		if !matches!(self.peek()?, ';') {
			self.push_back(type_char);
			return self.check_depth(|v| Ok(v.continue_parse_list()?.into()));
		}

		self.next();
		self.skip_whitespace();

		while !matches!(self.peek()?, ']') {
			let value = self.parse_element()?;

			match (&mut values, value) {
				(Value::ByteArray(l), Value::Byte(v)) => l.push(v),
				(Value::IntArray(l), Value::Int(v)) => l.push(v),
				(Value::LongArray(l), Value::Long(v)) => l.push(v),
				_ => {
					self.fault(SnbtErrorType::WrongTypeInArray)?;
				}
			}

			self.skip_whitespace();
			if matches!(self.peek()?, ',') {
				self.next();
				self.skip_whitespace();
			} else if !matches!(self.peek()?, ']') {
				self.fault(SnbtErrorType::ExpectComma)?;
			}
		}

		self.next();
		Ok(values)
	}

	fn parse_primitive(&mut self) -> Result<Value> {
		macro_rules! try_ret {
			($v:expr) => {{
				match $v {
					::std::result::Result::Ok(v) => return ::std::result::Result::Ok(v.into()),
					::std::result::Result::Err(_) => (),
				}
			}};
		}

		let target = self.read_unquoted_string()?;

		match target
			.bytes()
			.last()
			.ok_or_else(|| self.make_error(SnbtErrorType::ExpectValue))?
		{
			b'b' | b'B' => try_ret!(target[..target.len() - 1].parse::<i8>()),
			b's' | b'S' => try_ret!(target[..target.len() - 1].parse::<i16>()),
			b'l' | b'L' => try_ret!(target[..target.len() - 1].parse::<i64>()),
			b'f' | b'F' => try_ret!(target[..target.len() - 1].parse::<f32>()),
			b'd' | b'D' => try_ret!(target[..target.len() - 1].parse::<f64>()),
			_ => (),
		}

		match target.as_str() {
			"true" => return Ok(Value::Byte(1)),
			"false" => return Ok(Value::Byte(0)),
			_ => {
				try_ret!(target.parse::<i32>());
				try_ret!(target.parse::<f64>());
			}
		}

		if target.len() > STRING_MAX_LENGTH {
			self.fault(SnbtErrorType::LongString)?;
		}

		Ok(Value::String(target))
	}

	pub fn parse_element(&mut self) -> Result<Value> {
		self.skip_whitespace();

		match self.peek()? {
			'{' => self.check_depth(|v| Ok(v.parse_compound()?.into())),
			'[' => self.parse_list_like(),
			'"' | '\'' => self.read_quoted_string().map(Into::into),
			_ => self.parse_primitive(),
		}
	}

	pub fn read(&mut self) -> Result<Value> {
		let value = self.parse_element()?;

		self.skip_whitespace();
		if self.peek().is_ok() {
			self.fault(SnbtErrorType::TrailingData)?;
		}

		Ok(value)
	}

	#[must_use]
	pub const fn bytes_read(&self) -> usize {
		self.index
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct SnbtWriter<'a> {
	output: &'a mut String,
}

impl<'a> SnbtWriter<'a> {
	pub const fn new(output: &'a mut String) -> Self {
		Self { output }
	}

	fn write_str(&mut self, s: &str) {
		let mut need_quote = false;
		for c in s.chars() {
			if !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '+' | '.') {
				need_quote = true;
				break;
			}
		}

		if need_quote {
			self.output.push('"');
			for c in s.chars() {
				match c {
					'"' => self.output.push_str("\\\""),
					'\\' => self.output.push_str("\\\\"),
					_ => self.output.push(c),
				}
			}

			self.output.push('"');
		} else {
			self.output.push_str(s);
		}
	}

	fn write_primitive_array<'b>(
		&mut self,
		prefix: &str,
		iter: impl IntoIterator<Item = &'b (impl Into<Value> + 'b + Copy)>,
	) {
		self.output.push('[');
		self.output.push_str(prefix);

		let mut first = true;

		for v in iter {
			if !first {
				self.output.push(',');
			}

			first = false;
			self.write_element(&(*v).into());
		}

		self.output.push(']');
	}

	fn write_primitive(&mut self, postfix: &str, value: impl ToString) {
		self.output.push_str(&value.to_string());
		self.output.push_str(postfix);
	}

	fn write_list(&mut self, list: &List) {
		macro_rules! variant_impl {
			($v:expr, $handle:expr) => {{
				self.output.push('[');

				let mut first = true;
				for v in $v.iter() {
					if !first {
						self.output.push(',');
					}

					first = false;
					$handle(v);
				}

				self.output.push(']')
			}};
		}

		match list {
			List::Byte(v) => variant_impl!(v, |v| self.write_primitive("b", v)),
			List::Short(v) => variant_impl!(v, |v| self.write_primitive("s", v)),
			List::Int(v) => variant_impl!(v, |v| self.write_primitive("", v)),
			List::Long(v) => variant_impl!(v, |v| self.write_primitive("l", v)),
			List::Float(v) => variant_impl!(v, |v| self.write_primitive("f", v)),
			List::Double(v) => variant_impl!(v, |v| self.write_primitive("d", v)),
			List::ByteArray(v) => {
				variant_impl!(v, |v: &Vec<i8>| self.write_primitive_array("B", v));
			}
			List::IntArray(v) => {
				variant_impl!(v, |v: &Vec<i32>| self.write_primitive_array("", v));
			}
			List::LongArray(v) => {
				variant_impl!(v, |v: &Vec<i64>| self.write_primitive_array("L", v));
			}
			List::String(v) => variant_impl!(v, |v| self.write_str(v)),
			List::List(v) => variant_impl!(v, |v| self.write_list(v)),
			List::Compound(v) => variant_impl!(v, |v| self.write_compound(v)),
			List::End => self.output.push_str("[]"),
		}
	}

	fn write_compound(&mut self, compound: &Compound) {
		self.output.push('{');

		let mut first = true;
		for (k, v) in compound {
			if !first {
				self.output.push(',');
			}

			first = false;

			self.write_str(k);
			self.output.push(':');
			self.write_element(v);
		}

		self.output.push('}');
	}

	pub fn write_element(&mut self, value: &Value) {
		match value {
			Value::Byte(v) => self.write_primitive("b", v),
			Value::Short(v) => self.write_primitive("s", v),
			Value::Int(v) => self.write_primitive("", v),
			Value::Long(v) => self.write_primitive("l", v),
			Value::Float(v) => self.write_primitive("f", v),
			Value::Double(v) => self.write_primitive("d", v),
			Value::ByteArray(v) => self.write_primitive_array("B;", v),
			Value::IntArray(v) => self.write_primitive_array("I;", v),
			Value::LongArray(v) => self.write_primitive_array("L;", v),
			Value::String(v) => self.write_str(v),
			Value::List(v) => self.write_list(v),
			Value::Compound(v) => self.write_compound(v),
		}
	}
}

impl Display for SnbtWriter<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self.output)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnbtError {
	pub kind: SnbtErrorType,
	pub line: usize,
	pub column: usize,
}

impl SnbtError {
	#[must_use]
	pub const fn new(kind: SnbtErrorType, line: usize, column: usize) -> Self {
		Self { kind, line, column }
	}
}

impl Display for SnbtError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("@ ")?;
		Display::fmt(&self.line, f)?;
		f.write_char(',')?;
		Display::fmt(&self.column, f)?;
		f.write_str(": ")?;
		Display::fmt(&self.kind, f)
	}
}

impl Error for SnbtError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnbtErrorType {
	ReachEndOfStream,
	InvalidEscapeSequence,
	EmptyKeyInCompound,
	ExpectColon,
	ExpectValue,
	ExpectComma,
	WrongTypeInArray,
	DifferentTypesInList,
	LongString,
	TrailingData,
	DepthLimitExceeded,
}

impl Display for SnbtErrorType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::ReachEndOfStream => "reach end of stream",
			Self::InvalidEscapeSequence => "invalid escape sequence",
			Self::EmptyKeyInCompound => "empty key in compound",
			Self::ExpectColon => "expect colon",
			Self::ExpectValue => "expect value",
			Self::ExpectComma => "expect comma",
			Self::WrongTypeInArray => "wrong type in array",
			Self::DifferentTypesInList => "different types in list",
			Self::LongString => "long string",
			Self::TrailingData => "extra data after end",
			Self::DepthLimitExceeded => "depth limit exceeded",
		})
	}
}

type Result<T> = super::Result<T, SnbtError>;

pub fn from_snbt_str(snbt: &str) -> Result<Value> {
	SnbtReader::new(snbt).read()
}

#[must_use]
pub fn to_snbt_string(value: &Value) -> String {
	let mut output = String::new();
	let mut writer = SnbtWriter::new(&mut output);

	writer.write_element(value);

	output
}

#[cfg(test)]
mod tests {
	#[cfg(feature = "preserve_order")]
	use super::to_snbt_string;
	use super::{MAX_DEPTH, Result, SnbtErrorType, from_snbt_str};
	use crate::{List, Value};

	#[test]
	fn parse() -> Result<()> {
		let str = r#"
                            {
                                foo: 1,
                                'bar': 1.0,
                                "baz": 1.0f,
                                "hello'": "hello world",
                                "world": "hello\"world",
                                1.5f: 1.5d,
                                3b: 2f,
                                bool: false,
                                more: {
                                    iarr: [I; 1, 2, 3],
                                    larr: [L; 1L, 2L, 3L],
                                },
                                empty: [Bibabo ],
                            }
                        "#;

		let value = from_snbt_str(str)?;
		let Value::Compound(cpd) = &value else {
			unreachable!()
		};

		assert_eq!(*cpd.get("foo").unwrap(), 1_i32.into());
		assert_eq!(*cpd.get("bar").unwrap(), 1_f64.into());
		assert_eq!(*cpd.get("baz").unwrap(), 1_f32.into());
		assert_eq!(*cpd.get("hello'").unwrap(), "hello world".into());
		assert_eq!(*cpd.get("world").unwrap(), "hello\"world".into());
		assert_eq!(*cpd.get("1.5f").unwrap(), 1.5_f64.into());
		assert_eq!(*cpd.get("3b").unwrap(), 2_f32.into());
		assert_eq!(*cpd.get("bool").unwrap(), 0_i8.into());

		let Some(Value::Compound(more)) = cpd.get("more") else {
			unreachable!()
		};

		assert_eq!(*more.get("iarr").unwrap(), vec![1, 2, 3].into());

		assert_eq!(*more.get("larr").unwrap(), vec![1_i64, 2, 3].into());

		let Value::List(List::String(list)) = cpd.get("empty").unwrap() else {
			unreachable!()
		};

		assert_eq!(list[0], "Bibabo");

		assert_eq!(
			from_snbt_str("\"\\n\"").unwrap_err().kind,
			SnbtErrorType::InvalidEscapeSequence
		);

		assert_eq!(
			from_snbt_str("[L; 1]").unwrap_err().kind,
			SnbtErrorType::WrongTypeInArray
		);

		assert_eq!(
			from_snbt_str("[L; 1L, 2L, 3L").unwrap_err().kind,
			SnbtErrorType::ReachEndOfStream
		);

		assert_eq!(
			from_snbt_str("[L; 1L, 2L, 3L,]dewdwe").unwrap_err().kind,
			SnbtErrorType::TrailingData
		);

		assert_eq!(
			from_snbt_str("{ foo: }").unwrap_err().kind,
			SnbtErrorType::ExpectValue
		);

		assert_eq!(
			from_snbt_str("{ {}, }").unwrap_err().kind,
			SnbtErrorType::EmptyKeyInCompound
		);

		assert_eq!(
			from_snbt_str("{ foo 1 }").unwrap_err().kind,
			SnbtErrorType::ExpectColon
		);

		assert_eq!(
			from_snbt_str("{ foo: 1 bar: 2 }").unwrap_err().kind,
			SnbtErrorType::ExpectComma
		);

		assert_eq!(
			from_snbt_str("[{}, []]").unwrap_err().kind,
			SnbtErrorType::DifferentTypesInList
		);

		assert_eq!(
			from_snbt_str(&String::from_utf8(vec![b'e'; 32768]).unwrap())
				.unwrap_err()
				.kind,
			SnbtErrorType::LongString
		);

		assert_eq!(
			from_snbt_str(
				&String::from_utf8([[b'['; MAX_DEPTH + 1], [b']'; MAX_DEPTH + 1]].concat())
					.unwrap()
			)
			.unwrap_err()
			.kind,
			SnbtErrorType::DepthLimitExceeded
		);

		#[cfg(feature = "preserve_order")]
		assert_eq!(
			to_snbt_string(&value),
			r#"{foo:1,bar:1d,baz:1f,"hello'":"hello world",world:"hello\"world",1.5f:1.5d,3b:2f,bool:0b,more:{iarr:[I;1,2,3],larr:[L;1l,2l,3l]},empty:[Bibabo]}"#
		);

		Ok(())
	}
}
