mod word;

use std::ops::{Deref, DerefMut};

use thiserror::Error;

pub use self::word::ProgramWord;
use super::{Instr, InstrDecodingError, ToLasm};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Program(Vec<ProgramWord>);

impl Program {
	#[must_use]
	pub const fn new() -> Self {
		Self(Vec::new())
	}

	pub fn prepend(&mut self, instr: ProgramWord) {
		self.insert(0, instr);
	}

	pub fn prepend_many(&mut self, instrs: &[ProgramWord]) {
		let tail = self.len() - instrs.len();

		self.0.extend(instrs);
		self.0[instrs.len()..].rotate_left(tail);
	}

	pub fn decode(prog: &[u8], forbid_raw: bool) -> Result<Self, ProgramDecodingError> {
		if !matches!(prog.len() % 4, 0) {
			return Err(ProgramDecodingError::new(
				0,
				InstrDecodingError::SourceNotMultipleBytesOf4Bytes,
			));
		}

		let mut out = Vec::new();

		for i in 0..prog.len() / 4 {
			let bytes = [
				prog[i * 4],
				prog[i * 4 + 1],
				prog[i * 4 + 2],
				prog[i * 4 + 3],
			];

			let pword = match Instr::decode(bytes) {
				Ok(instr) => ProgramWord::Instr(instr),
				Err(err) if forbid_raw => return Err(ProgramDecodingError::new(i, err)),
				Err(_) => ProgramWord::Raw(bytes),
			};

			out.push(pword);
		}

		Ok(Self(out))
	}

	#[must_use]
	pub fn to_folded_bytes(&self) -> Vec<[u8; 4]> {
		self.iter().copied().map(ProgramWord::encode).collect()
	}

	#[must_use]
	pub fn encode(&self) -> Vec<u8> {
		let mut output = Vec::new();

		for pword in &**self {
			output.extend_from_slice(&pword.encode());
		}

		output
	}

	pub fn encode_words(&self) -> Vec<u32> {
		self.iter().copied().map(ProgramWord::encode_word).collect()
	}

	#[must_use]
	pub fn to_lasm(&self, annotate_instr_addr: bool) -> String {
		if annotate_instr_addr {
			self.to_lasm_lines_annotated()
		} else {
			self.to_lasm_lines()
		}
		.join("\n")
	}

	#[must_use]
	pub fn to_lasm_lines(&self) -> Vec<String> {
		self.iter()
			.map(|pword| pword.to_lasm().into_owned())
			.collect()
	}

	#[must_use]
	pub fn to_lasm_lines_annotated(&self) -> Vec<String> {
		let mut counter = 0;
		self.iter()
			.map(|pword| {
				let instr = format!("{counter:#010X}: {}", pword.to_lasm());
				counter += 4;
				instr
			})
			.collect()
	}
}

impl Deref for Program {
	type Target = Vec<ProgramWord>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Program {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Extend<Instr> for Program {
	fn extend<T: IntoIterator<Item = Instr>>(&mut self, iter: T) {
		self.0.extend(iter.into_iter().map(ProgramWord::from));
	}
}

impl Extend<ProgramWord> for Program {
	fn extend<T: IntoIterator<Item = ProgramWord>>(&mut self, iter: T) {
		self.0.extend(iter);
	}
}

impl FromIterator<ProgramWord> for Program {
	fn from_iter<T: IntoIterator<Item = ProgramWord>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl FromIterator<Instr> for Program {
	fn from_iter<T: IntoIterator<Item = Instr>>(iter: T) -> Self {
		Self(iter.into_iter().map(ProgramWord::from).collect())
	}
}

#[derive(Debug, Error)]
#[error("{source}")]
pub struct ProgramDecodingError {
	#[source]
	source: InstrDecodingError,
	line: usize,
}

impl ProgramDecodingError {
	#[must_use]
	pub const fn source(&self) -> &InstrDecodingError {
		&self.source
	}

	#[must_use]
	pub const fn line(&self) -> usize {
		self.line
	}

	const fn new(line: usize, source: InstrDecodingError) -> Self {
		Self { source, line }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::asm::{Reg, RegOrLit2, cst};

	fn prog() -> Program {
		[
			Instr::Add(Reg::A0, 0xFFu8.into()),
			Instr::Sub(Reg::A0, 0xFFu8.into()),
			Instr::Div(Reg::A0, 0x00u8.into(), cst::DIV_ZRO_MIN.into()),
			Instr::Mod(
				Reg::A0,
				0x00u8.into(),
				(cst::DIV_ZRO_MIN | cst::DIV_OFW_MAX).into(),
			),
			Instr::Jpr(RegOrLit2::from(-80i16)),
		]
		.into_iter()
		.collect()
	}

	const fn encoded() -> [u8; 20] {
		[
			0x1C, 0x00, 0x00, 0xFF, 0x24, 0x00, 0x00, 0xFF, 0x34, 0x00, 0x00, 0x04, 0x3C, 0x00,
			0x00, 0x07, 0x70, 0xFF, 0xB0, 0x00,
		]
	}

	const fn assembled() -> [&'static str; 5] {
		[
			"add a0, 0xFF",
			"sub a0, 0xFF",
			"div a0, 0x0, DIV_ZRO_MIN",
			"mod a0, 0x0, DIV_ZRO_MIN | DIV_OFW_MAX",
			"jpr -0x50",
		]
	}

	#[test]
	fn encoding() {
		assert_eq!(prog().encode(), encoded(), "encoded program is not valid");
	}

	#[test]
	fn decoding() -> Result<(), ProgramDecodingError> {
		let re_prog = Program::decode(&encoded(), false)?;
		assert_eq!(
			re_prog,
			prog(),
			"original and re-encoded program are different"
		);

		Ok(())
	}

	#[test]
	fn asm_conversion() {
		let lasm = prog().to_lasm_lines();
		let assembled = assembled();

		assert_eq!(
			lasm.len(),
			assembled.len(),
			"assembled code is {} than expected.\nexpected:\n\n{}\n\ngot:\n\n{}",
			if lasm.len() > assembled.len() {
				"greater"
			} else {
				"smaller"
			},
			assembled.join("\n"),
			lasm.join("\n")
		);

		for i in 0..lasm.len() {
			assert_eq!(
				lasm[i],
				assembled[i],
				"assembled program differs from expected one.\nat line {}",
				i + 1,
			);
		}
	}
}
