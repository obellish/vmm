mod program_utils;

use program_utils::{Result, run_program};
use vmm::tape::{BoxTape, PtrTape, Tape, VecTape};

const PROGRAM: &str = include_str!("../programs/hello_world_test.bf");

fn run<T: Tape>(opt: bool) -> Result<()> {
	assert_eq!(run_program::<T>(PROGRAM, opt)?, b"Hello World! 255\n");

	Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn unoptimized_box_tape() -> Result<()> {
	run::<BoxTape>(false)
}

#[test]
#[cfg_attr(miri, ignore)]
fn unoptimized_ptr_tape() -> Result<()> {
	run::<PtrTape>(false)
}

#[test]
#[cfg_attr(miri, ignore)]
fn unoptimized_vec_tape() -> Result<()> {
	run::<VecTape>(false)
}

#[test]
#[cfg_attr(miri, ignore)]
fn optimized_box_tape() -> Result<()> {
	run::<BoxTape>(true)
}

#[test]
#[cfg_attr(miri, ignore)]
fn optimized_ptr_tape() -> Result<()> {
	run::<PtrTape>(true)
}

#[test]
#[cfg_attr(miri, ignore)]
fn optimized_vec_tape() -> Result<()> {
	run::<VecTape>(true)
}
