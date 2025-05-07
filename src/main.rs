use std::{fmt::Debug, fs, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::Result;
use serde::Serialize;
use tracing::warn;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use vmm::{Instruction, Optimizer, Program, Scanner, Vm};

fn main() -> Result<()> {
	install_tracing();
	color_eyre::install()?;

	let args = Args::parse();

	let raw_data = fs::read_to_string(args.file)?;

	let filtered_data = raw_data
		.chars()
		.filter(|c| matches!(c, '+' | '-' | '>' | '<' | ',' | '.' | '[' | ']'))
		.collect::<String>();

	let program = {
		let unoptimized = Scanner::new(&filtered_data).scan()?.collect::<Program>();

		serialize_and_write(&unoptimized, "unoptimized_program")?;

		write_program(&unoptimized, "unoptimized")?;
		if args.optimize {
			let mut optimizer = Optimizer::new(unoptimized.clone());

			let optimized = optimizer.optimize()?;

			serialize_and_write(&optimized, "optimized_program")?;

			write_program(&optimized, "optimized")?;

			if program_to_string(&unoptimized) != program_to_string(&optimized) {
				warn!("program instructions do not match, semantics may be different");
			}

			optimized
		} else {
			unoptimized
		}
	};

	let profiler = if program.needs_input() {
		let mut vm = Vm::stdio(program).and_profile();

		vm.run()?;

		vm.profiler()
	} else {
		let mut vm = Vm::stdio(program)
			.with_input(std::io::empty())
			.and_profile();

		vm.run()?;

		vm.profiler()
	};

	serialize_and_write(&profiler, "profiler")?;

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	pub file: PathBuf,
	#[arg(short, long)]
	pub optimize: bool,
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

	let filter_layer = EnvFilter::new("info");
	let fmt_layer = fmt::layer().with_target(false).with_filter(filter_layer);

	tracing_subscriber::registry()
		.with(file_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();
}

fn serialize_and_write<S: Serialize>(p: &S, name: &str) -> Result<()> {
	fs::write(
		format!("./out/{name}.json"),
		serde_json::to_string_pretty(p)?,
	)?;

	fs::write(
		format!("./out/{name}.ron"),
		ron::ser::to_string_pretty(p, ron::ser::PrettyConfig::new())?,
	)?;

	Ok(())
}

fn write_program(p: &[Instruction], name: &str) -> Result<()> {
	fs::write(format!("./out/{name}.bf"), program_to_string(p))?;

	Ok(())
}

fn program_to_string(p: &[Instruction]) -> String {
	p.iter().map(ToString::to_string).collect::<String>()
}
