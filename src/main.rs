use std::{fmt::Debug, fs, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
	EnvFilter,
	fmt::{self, format::FmtSpan},
	prelude::*,
};
use vmm_interpret::{Interpreter, Profiler};
use vmm_opt::{HashMetadataStore, Optimizer, OutputMetadataStore};
use vmm_parse::Parser as BfParser;
// use vmm::{MapStore, Optimizer, Profiler, Program, RonWrapperStore, Scanner, Vm};
use vmm_program::Program;

fn main() -> Result<()> {
	_ = fs::remove_dir_all("./out");

	fs::create_dir_all("./out")?;
	install_tracing();
	color_eyre::install()?;

	let args = Args::parse();

	let raw_data = fs::read_to_string(args.file)?;

	let filtered_data = raw_data
		.chars()
		.filter(|c| matches!(c, '+' | '-' | '>' | '<' | ',' | '.' | '[' | ']'))
		.collect::<String>();

	let program = {
		let unoptimized = BfParser::new(&filtered_data).scan()?.collect::<Program>();
		if args.optimize {
			let mut optimizer = Optimizer::new(
				unoptimized,
				OutputMetadataStore::new(HashMetadataStore::new(), PathBuf::new().join("./out"))?,
			);

			optimizer.optimize()?
		} else {
			unoptimized
		}
	};

	let profiler = if program.needs_input() {
		let mut vm = Interpreter::stdio(program).and_with_profiler();

		vm.run()?;

		vm.profiler()
	} else {
		let mut vm = Interpreter::stdio(program)
			.with_input(std::io::empty())
			.and_with_profiler();

		vm.run()?;

		vm.profiler()
	};

	write_profiler(profiler)?;

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

	let json_log_file = fs::OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open("./out/output.json")
		.expect("failed to open file");

	let file_layer = fmt::layer()
		.with_span_events(FmtSpan::CLOSE)
		.with_ansi(false)
		.with_writer(log_file);

	let filter_layer = EnvFilter::new("info");
	let fmt_layer = fmt::layer().with_target(false).with_filter(filter_layer);

	let json_file_layer = fmt::layer()
		.with_ansi(false)
		.json()
		.flatten_event(false)
		.with_span_events(FmtSpan::FULL)
		.with_writer(json_log_file);

	tracing_subscriber::registry()
		.with(json_file_layer)
		.with(file_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();
}

fn write_profiler(p: Profiler) -> Result<()> {
	fs::write(
		"./out/profiler.ron",
		ron::ser::to_string_pretty(&p, ron::ser::PrettyConfig::new())?,
	)?;

	Ok(())
}
