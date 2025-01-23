use customasm::{
	asm::{self, AssemblyOptions},
	diagn::Report,
	util::FileServerMock,
};

use super::{
	asm::{Program, ProgramDecodingError},
	bytes::{bytes_to_words, words_to_bytes},
};

static CUSTOMASM_HEADER: &str = include_str!("customasm.def");

pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
	let mut src = String::from("#include \"header.lasm\"");
	src.push('\n');
	src.push_str(source);

	let mut fileserver = FileServerMock::new();
	fileserver.add("header.lasm", CUSTOMASM_HEADER);
	fileserver.add("src.lasm", src);

	let opts = AssemblyOptions::new();
	let mut report = Report::new();

	let assembly = asm::assemble(&mut report, &opts, &mut fileserver, &["src.lasm"]);

	match assembly.output {
		Some(output) if !report.has_errors() && !assembly.error => Ok(output.format_binary()),
		Some(_) | None => {
			let mut err = Vec::new();
			report.print_all(&mut err, &fileserver, false);
			Err(String::from_utf8_lossy(&err).into_owned())
		}
	}
}

pub fn assemble_words(source: &str) -> Result<Vec<u32>, String> {
	assemble(source).map(|words| bytes_to_words(&words))
}

pub fn assemble_program(source: &str) -> Result<Result<Program, ProgramDecodingError>, String> {
	Ok(Program::decode(&assemble(source)?, false))
}

pub fn disassemble(code: &[u8], annotate_instr_addr: bool) -> Result<String, ProgramDecodingError> {
	Ok(Program::decode(code, false)?.to_lasm(annotate_instr_addr))
}

pub fn disassemble_words(
	code: &[u32],
	annotate_instr_addr: bool,
) -> Result<String, ProgramDecodingError> {
	Ok(Program::decode(&words_to_bytes(code), false)?.to_lasm(annotate_instr_addr))
}

#[cfg(test)]
mod tests {
	use super::*;

	static DEMO_ASM: &str = include_str!("demo.lasm");

	#[test]
	fn works() -> Result<(), String> {
		let asm_bytes = assemble(DEMO_ASM)?;

		assert_eq!(asm_bytes.len() % 4, 0, "unaligned assembly output");

		assert_eq!(
			asm_bytes,
			[
				0x1C, 0x00, 0x00, 0xFF, 0x24, 0x00, 0x00, 0xFF, 0x34, 0x00, 0x00, 0x04, 0x3C, 0x00,
				0x00, 0x04, 0x70, 0xFF, 0xF0, 0x00,
			],
			"bad assembly output"
		);

		Ok(())
	}
}
