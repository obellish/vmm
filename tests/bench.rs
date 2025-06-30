mod program_utils;

use program_utils::{Result, run_program};
use vmm::tape::{BoxTape, PtrTape, StackTape, Tape, VecTape};

const PROGRAM: &str = include_str!("../programs/bench.bf");

fn run<T: Tape>(opt: bool) -> Result<()> {
	assert_eq!(
		run_program::<T>(PROGRAM, opt)?,
		b"ZYXWVUTSRQPONMLKJIHGFEDCBA\n"
	);

	Ok(())
}

#[test]
#[ignore = "takes too long"]
fn unoptimized_box_tape() -> Result<()> {
	run::<BoxTape>(false)
}

#[test]
#[ignore = "takes too long"]
fn unoptimized_ptr_tape() -> Result<()> {
	run::<PtrTape>(false)
}

#[test]
#[ignore = "takes too long"]
fn unoptimized_vec_tape() -> Result<()> {
	run::<VecTape>(false)
}

#[test]
#[ignore = "takes too long"]
fn unoptimized_stack_tape() -> Result<()> {
	run::<StackTape>(false)
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

#[test]
#[cfg_attr(miri, ignore)]
fn optimized_stack_tape() -> Result<()> {
	run::<StackTape>(true)
}
