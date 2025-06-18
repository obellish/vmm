#[doc(inline)]
pub use {
	vmm_alloc as alloc, vmm_interpret as interpret, vmm_ir as ir, vmm_opt as opt,
	vmm_parse as parse, vmm_program as program, vmm_utils as utils,
};

#[cfg(test)]
mod tests {
	use std::io;

	use super::{
		interpret::Interpreter,
		opt::{HashMetadataStore, Optimizer},
		parse::{ParseError, Parser},
		program::Program,
	};

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
	#[cfg_attr(miri, ignore)]
	fn hello_world_optimized() -> anyhow::Result<()> {
		assert_eq!(run_program(HELLO_WORLD, true)?, b"Hello World!\n");

		Ok(())
	}

	#[test]
	#[cfg_attr(miri, ignore)]
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
