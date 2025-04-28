use std::{fmt::Debug, fs, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use vmm::{ExecutionUnit, Optimizer, Program, Scanner, Vm};

fn main() -> Result<()> {
	install_tracing();
	color_eyre::install()?;

	let args = Args::parse();

	let raw_data = fs::read_to_string(args.file)?;

	let filtered_data = raw_data
		.chars()
		.filter(|c| matches!(c, '+' | '-' | '>' | '<' | ',' | '.' | '[' | ']'))
		.collect::<String>();

	let execution_unit = Scanner::new(&filtered_data).collect::<ExecutionUnit>();

	serialize_unit(&execution_unit, false)?;

	let optimized = Optimizer::new(execution_unit, true).optimize()?;

	serialize_unit(&optimized, true)?;

	let vm = if optimized.program().needs_input() {
		Vm::stdio(optimized).into_dyn()
	} else {
		Vm::stdio(optimized).with_input(std::io::empty()).into_dyn()
	};

	vm.run()?;

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	pub file: PathBuf,
}

fn install_tracing() {
	fs::create_dir_all("./out").unwrap();

	let log_file = fs::OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open("./out/output.log")
		.expect("failed to open file");

	let file_layer = fmt::layer().with_ansi(false).with_writer(log_file);

	let filter_layer = EnvFilter::new("debug");
	let fmt_layer = fmt::layer().with_target(false).with_filter(filter_layer);

	tracing_subscriber::registry()
		.with(file_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();
}

fn serialize_unit(p: &ExecutionUnit, optimized: bool) -> Result<()> {
	fs::write(
		format!(
			"./out/{}.json",
			if optimized {
				"optimized"
			} else {
				"unoptimized"
			}
		),
		serde_json::to_string_pretty(p)?,
	)?;

	Ok(())
}
