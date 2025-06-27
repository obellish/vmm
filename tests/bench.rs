mod program_utils;

use program_utils::{Result, run_program};
use vmm::tape::{BoxTape, PtrTape, Tape, VecTape};

const PROGRAM: &str = include_str!("../programs/bench.bf");

fn run<T: Tape>(opt: bool) -> Result<()> {
	assert_eq!(
		run_program::<T>(PROGRAM, opt)?,
		b"ZYXWVUTSRQPONMLKJIHGFEDCBA\n"
	);

	Ok(())
}

#[test]
fn unoptimized_box_tape() -> Result<()> {
	run::<BoxTape>(false)
}

#[test]
fn unoptimized_ptr_tape() -> Result<()> {
	run::<PtrTape>(false)
}

#[test]
fn unoptimized_vec_tape() -> Result<()> {
	run::<VecTape>(false)
}

#[test]
fn optimized_box_tape() -> Result<()> {
	run::<BoxTape>(true)
}

#[test]
fn optimized_ptr_tape() -> Result<()> {
	run::<PtrTape>(true)
}

#[test]
fn optimized_vec_tape() -> Result<()> {
	run::<VecTape>(true)
}
