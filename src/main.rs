use std::{alloc::System, path::PathBuf};

use clap::Parser;
use color_eyre::{
	Section,
	eyre::{Report, Result},
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use vmm::{Chunk, OpCode, Vm};
use vmm_alloc::{AllocChain, UnsafeStalloc};

#[global_allocator]
static GLOBAL: AllocChain<'static, UnsafeStalloc<1024, 4>, System> =
	unsafe { UnsafeStalloc::new().chain(&System) };

fn main() -> Result<()> {
	install_tracing();
	color_eyre::install()?;

	let args = Args::parse();

	let file_content = read_file(args)?;

	let mut c = Chunk::new();
	let mut constant = c.push_constant(1.2);
	c.push(OpCode::Constant(constant), 123);

	constant = c.push_constant(3.4);
	c.push(OpCode::Constant(constant), 123);

	c.push(OpCode::Add, 123);

	constant = c.push_constant(5.6);

	c.push(OpCode::Constant(constant), 123);

	c.push(OpCode::Divide, 123);
	c.push(OpCode::Negate, 123);

	c.push(OpCode::Return, 123);

	let mut vm = Vm::with_chunk(c);

	vm.interpret(file_content)?;

	println!("{vm:?}");

	Ok(())
}

fn install_tracing() {
	let fmt_layer = fmt::layer().with_target(false);
	let filter_layer = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new("info"))
		.unwrap();

	tracing_subscriber::registry()
		.with(filter_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();
}

#[tracing::instrument]
fn read_file(args: Args) -> Result<String, Report> {
	std::fs::read_to_string(args.file).map_err(|e| {
		let report: Report = e.into();

		report
			.wrap_err("Unable to read source file")
			.suggestion("try using a file that exists next time")
	})
}

#[derive(Debug, Parser)]
struct Args {
	pub file: PathBuf,
}
