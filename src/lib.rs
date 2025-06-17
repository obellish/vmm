//! This is only for the tests

#[cfg(test)]
mod tests {
	use std::io;

	use vmm_interpret::Interpreter;
	use vmm_opt::{HashMetadataStore, Optimizer};
	use vmm_parse::{ParseError, Parser};
	use vmm_program::Program;

	const HELLO_WORLD: &str = include_str!("../programs/hello_world.bf");

	const HELLO_WORLD_TEST: &str = include_str!("../programs/hello_world_test.bf");

	const BENCH: &str = include_str!("../programs/bench.bf");

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
		assert_eq!(run_program(HELLO_WORLD, false)?, b"Hello World!\n");

		Ok(())
	}

	#[test]
	fn hello_world_optimized() -> anyhow::Result<()> {
		assert_eq!(run_program(HELLO_WORLD, true)?, b"Hello World!\n");

		Ok(())
	}

	#[test]
	fn hello_world_test_unoptimized() -> anyhow::Result<()> {
		assert_eq!(run_program(HELLO_WORLD_TEST, false)?, b"Hello World! 255\n");

		Ok(())
	}

	#[test]
	#[cfg_attr(miri, ignore)]
	fn hello_world_test_optimized() -> anyhow::Result<()> {
		assert_eq!(run_program(HELLO_WORLD_TEST, true)?, b"Hello World! 255\n");

		Ok(())
	}

	// We only run optimized bc unoptimized takes too long
	#[test]
	#[cfg_attr(miri, ignore)]
	fn bench_optimized() -> anyhow::Result<()> {
		assert_eq!(run_program(BENCH, true)?, b"ZYXWVUTSRQPONMLKJIHGFEDCBA\n");

		Ok(())
	}
}
