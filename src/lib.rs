//! This is only for the tests

#[cfg(test)]
mod tests {
	use std::io;

	use vmm_interpret::Interpreter;
	use vmm_opt::{HashMetadataStore, Optimizer};
	use vmm_parse::{ParseError, Parser};
	use vmm_program::Program;

	const HELLO_WORLD_RAW: &str = include_str!("../programs/hello_world.bf");

	fn get_program(raw: &'static str) -> Result<Program, ParseError> {
		Parser::new(raw)
			.scan()
			.map(|v| v.into_iter().collect::<Program>())
	}

	fn run_program(program: &'static str, optimized: bool) -> anyhow::Result<Vec<u8>> {
		let program = get_program(program)?;

		let output = {
			let program = if optimized {
				Optimizer::new(program, HashMetadataStore::new()).optimize()?
			} else {
				program
			};

			let mut interpreter = Interpreter::new(program, io::empty(), Vec::<u8>::new());

			interpreter.run()?;

			interpreter.output().clone()
		};

		Ok(output)
	}

	#[test]
	fn hello_world_unoptimized() -> anyhow::Result<()> {
		assert_eq!(run_program(HELLO_WORLD_RAW, false)?, b"Hello World!\n");

		Ok(())
	}

	#[test]
	fn hello_world_optimized() -> anyhow::Result<()> {
		assert_eq!(run_program(HELLO_WORLD_RAW, true)?, b"Hello World!\n");

		Ok(())
	}
}
