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

	#[test]
	fn hello_world_unoptimized() -> anyhow::Result<()> {
		let program = get_program(HELLO_WORLD_RAW)?;

		let output = {
			let mut interpreter = Interpreter::new(program, io::empty(), Vec::<u8>::new());

			interpreter.run()?;

            interpreter.output().clone()
		};

        assert_eq!(output, b"Hello World!\n");

		Ok(())
	}

    #[test]
    fn hello_world_optimized() ->anyhow::Result<()> {
        let program = get_program(HELLO_WORLD_RAW)?;

        let program = Optimizer::new(program, HashMetadataStore::new()).optimize()?;

        let output = {
            let mut interpreter = Interpreter::new(program, io::empty(), Vec::<u8>::new());

            interpreter.run()?;

            interpreter.output().clone()
        };

        assert_eq!(output, b"Hello World!\n");

        Ok(())
    }
}
