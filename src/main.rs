use std::{
	alloc::System,
	fmt::{Display, Formatter, Result as FmtResult},
	fs,
	io::empty,
	path::PathBuf,
};

use clap::{Parser, ValueEnum, builder::PossibleValue};
use color_eyre::eyre::Result;
use tracing::{info, info_span};
use tracing_error::ErrorLayer;
use tracing_flame::FlameLayer;
use tracing_subscriber::{
	EnvFilter,
	fmt::{self, format::FmtSpan},
	prelude::*,
};
use vmm::{
	alloc::{AllocChain, UnsafeStalloc},
	alloc_stats::{Region, StatsAlloc},
	interpret::{Interpreter, Profiler},
	opt::{HashMetadataStore, Optimizer, OutputMetadataStore},
	parse::Parser as BfParser,
	program::Program,
	tape::PtrTape,
	utils::HeapSize as _,
};
use vmm_tape::{BoxTape, VecTape};

#[global_allocator]
static ALLOC: StatsAlloc<AllocChain<'static, UnsafeStalloc<65535, 4>, System>> =
	StatsAlloc::new(unsafe { UnsafeStalloc::new() }.chain(&System));

fn main() -> Result<()> {
	let mut region = Region::new(&ALLOC);

	_ = fs::remove_dir_all("./out");

	fs::create_dir_all("./out")?;
	let _guard = install_tracing();
	color_eyre::install()?;

	info_span!("after_install").in_scope(|| report_alloc_stats(&mut region));

	let args = match Args::try_parse() {
		Ok(args) => args,
		Err(e) => {
			eprintln!("{e}");
			return Ok(());
		}
	};

	region.reset();

	let raw_data = fs::read_to_string(args.file)?;

	let filtered_data = raw_data
		.chars()
		.filter(|c| matches!(c, '+'..='.' | '>' | '<' | '[' | ']'))
		.collect::<String>();

	info_span!("after_read_and_filter").in_scope(|| report_alloc_stats(&mut region));

	let program = {
		let unoptimized = BfParser::new(&filtered_data)
			.scan()?
			.into_iter()
			.collect::<Program>();

		info_span!("after_parse").in_scope(|| report_alloc_stats(&mut region));

		info!(
			"size of raw: {} bytes (len: {})",
			unoptimized.heap_size(),
			unoptimized.len()
		);
		if args.optimize {
			let mut optimizer = Optimizer::new(
				unoptimized,
				OutputMetadataStore::new(HashMetadataStore::new(), PathBuf::new().join("./out"))?,
			);

			let out = optimizer.optimize()?;

			info_span!("after optimize").in_scope(|| report_alloc_stats(&mut region));

			out
		} else {
			unoptimized
		}
	};

	info!(
		"size of final: {} bytes (len: {})",
		program.heap_size(),
		program.len()
	);

	let ir: String = program
		.iter()
		.map(|i| i.to_string() + "\n")
		.collect::<String>();

	fs::write("./out/ir.txt", ir)?;

	region.reset();

	let profiler = match (program.needs_input(), args.tape) {
		(true, TapeType::Ptr) => {
			let mut vm = Interpreter::<PtrTape>::stdio(program).and_with_profiler();

			vm.run()?;

			vm.profiler()
		}
		(true, TapeType::Box) => {
			let mut vm = Interpreter::<BoxTape>::stdio(program).and_with_profiler();

			vm.run()?;

			vm.profiler()
		}
		(true, TapeType::Vec) => {
			let mut vm = Interpreter::<VecTape>::stdio(program).and_with_profiler();

			vm.run()?;

			vm.profiler()
		}
		(false, TapeType::Ptr) => {
			let mut vm = Interpreter::<PtrTape>::stdio(program)
				.with_input(empty())
				.and_with_profiler();

			vm.run()?;

			vm.profiler()
		}
		(false, TapeType::Box) => {
			let mut vm = Interpreter::<BoxTape>::stdio(program)
				.with_input(empty())
				.and_with_profiler();

			vm.run()?;

			vm.profiler()
		}
		(false, TapeType::Vec) => {
			let mut vm = Interpreter::<VecTape>::stdio(program)
				.with_input(empty())
				.and_with_profiler();

			vm.run()?;

			vm.profiler()
		}
	};

	println!("\n");

	info_span!("after_run").in_scope(|| report_alloc_stats(&mut region));

	write_profiler(profiler)?;

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	pub file: PathBuf,
	#[arg(short, long)]
	pub optimize: bool,
	#[arg(short, long, default_value_t = TapeType::Ptr)]
	pub tape: TapeType,
}

#[derive(Debug, Clone, Copy)]
enum TapeType {
	Box,
	Vec,
	Ptr,
}

impl Display for TapeType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Box => "box",
			Self::Vec => "vec",
			Self::Ptr => "ptr",
		})
	}
}

impl ValueEnum for TapeType {
	fn value_variants<'a>() -> &'a [Self] {
		&[Self::Box, Self::Vec, Self::Ptr]
	}

	fn to_possible_value(&self) -> Option<PossibleValue> {
		Some(PossibleValue::new(match self {
			Self::Box => "box",
			Self::Vec => "vec",
			Self::Ptr => "ptr",
		}))
	}
}

fn install_tracing() -> impl Drop {
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

	let file_layer = fmt::layer().with_ansi(false).with_writer(log_file);

	let filter_layer = EnvFilter::new("info");
	let fmt_layer = fmt::layer().with_target(false).with_filter(filter_layer);

	let json_file_layer = fmt::layer()
		.with_ansi(false)
		.json()
		.flatten_event(true)
		.with_span_events(FmtSpan::FULL)
		.with_writer(json_log_file);

	let (flame_layer, guard) = FlameLayer::with_file("./out/output.folded").unwrap();

	tracing_subscriber::registry()
		.with(json_file_layer)
		.with(file_layer)
		.with(fmt_layer)
		.with(flame_layer)
		.with(ErrorLayer::default())
		.init();

	guard
}

fn write_profiler(p: Profiler) -> Result<()> {
	fs::write(
		"./out/profiler.ron",
		ron::ser::to_string_pretty(&p, ron::ser::PrettyConfig::new())?,
	)?;

	Ok(())
}

fn report_alloc_stats<T>(region: &mut Region<'_, T>) {
	info!("allocation stats");

	let stats = region.stat_diff();

	info!("allocations: {}", stats.allocations);
	info!("deallocations: {}", stats.deallocations);
	info!("reallocations: {}", stats.reallocations);
	info!("bytes allocated: {}", stats.bytes_allocated);
	info!("bytes deallocated: {}", stats.bytes_deallocated);
	info!("bytes reallocated: {}", stats.bytes_reallocated);

	region.reset();
}
