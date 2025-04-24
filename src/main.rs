#![expect(unused)]

use std::{alloc::System, fmt::Debug, fs, path::PathBuf};

use clap::Parser;
use color_eyre::{
	Section,
	eyre::{Report, Result},
};
use serde::Serialize;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use vmm::{Chunk, CompileError, Compiler, OpCode, Scanner, Vm};
use vmm_alloc::{AllocChain, UnsafeStalloc};

#[global_allocator]
static GLOBAL: AllocChain<'static, UnsafeStalloc<1024, 4>, System> =
	unsafe { UnsafeStalloc::new().chain(&System) };

fn main() -> Result<()> {
	install_tracing();
	color_eyre::install()?;

	let args = Args::parse();

	let file_content = read_file(args)?;

	let scanner = Scanner::new(&file_content);

	let compiler = scanner.collect::<Result<Compiler, CompileError>>()?;

	write_state(&compiler, "compiler")?;

	let chunk = compiler.compile()?;

	write_state(&chunk, "chunk")?;

	let mut vm = Vm::with_chunk(chunk);

	write_state(&vm, "vm")?;

	vm.interpret()?;

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	pub file: PathBuf,
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
fn read_file(args: Args) -> Result<String> {
	fs::read_to_string(args.file).map_err(|e| {
		let report: Report = e.into();

		report
			.wrap_err("Unable to read source file")
			.suggestion("try using a file that exists next time")
	})
}

#[tracing::instrument]
fn write_state<S>(s: &S, file_name: &str) -> Result<()>
where
	S: Debug + Serialize,
{
	fs::create_dir_all("./out")?;

	fs::write(
		format!("./out/{file_name}.json"),
		serde_json::to_string_pretty(s)?,
	)?;

	println!("{file_name}: {s:?}");

	Ok(())
}
