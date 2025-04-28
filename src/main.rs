use std::{alloc::System, fmt::Debug, fs, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use vmm::{ExecutionUnit, Optimizer, Scanner};
use vmm_alloc::{AllocChain, UnsafeStalloc};

#[global_allocator]
static GLOBAL: AllocChain<'static, UnsafeStalloc<1024, 4>, System> =
	unsafe { UnsafeStalloc::new().chain(&System) };

fn main() -> Result<()> {
	install_tracing();
	color_eyre::install()?;

	let args = Args::parse();

	let raw_data = fs::read_to_string(args.file)?;

	let execution_unit = Scanner::new(&raw_data).collect::<ExecutionUnit>();

	println!("{execution_unit:?}");

	let mut optimized = Optimizer::new(execution_unit).optimize()?;

	println!("{optimized:?}");

	optimized.run();

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
