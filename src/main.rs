#![allow(clippy::large_stack_frames)]

mod args;

#[cfg(any(miri, not(feature = "mimalloc")))]
use std::alloc::System as Alloc;
use std::{
	fs,
	io::{Stdout, empty, stdin, stdout},
	path::PathBuf,
};

use clap::Parser as _;
use color_eyre::eyre::Result;
use serde_binary::{Config, to_writer_with_config};
use serde_reflection::{Tracer, TracerConfig};
use tracing::{debug, debug_span, info};
use tracing_error::ErrorLayer;
use tracing_flame::FlameLayer;
use tracing_subscriber::{
	EnvFilter,
	fmt::{self, format::FmtSpan},
	prelude::*,
};
use vmm::{
	alloc_stats::{Region, StatsAlloc},
	interpret::{Interpreter, Profiler},
	ir::{BlockInstruction, Instruction, MinimumOutputs as _, ScaleAnd, SuperInstruction},
	opt::{HashMetadataStore, Optimizer, OutputMetadataStore},
	parse::Parser as BfParser,
	program::Program,
	tape::{BoxTape, PtrTape, StackTape, Tape, VecTape},
	utils::{CopyWriter, HeapSize as _},
};
#[cfg(all(not(miri), feature = "mimalloc"))]
use vmm_mimalloc::MiMalloc as Alloc;

use self::args::{Args, TapeType};

#[global_allocator]
static ALLOC: StatsAlloc<Alloc> = StatsAlloc::new(Alloc);

fn main() -> Result<()> {
	let mut region = Region::new(&ALLOC);
	let mut total = Region::new(&ALLOC);

	_ = fs::remove_dir_all("./out");

	fs::create_dir_all("./out")?;
	let _guard = install_tracing();
	color_eyre::install()?;
	write_format()?;

	debug_span!("after_install").in_scope(|| report_alloc_stats(&mut region));

	let Args {
		file,
		optimize,
		tape,
	} = match Args::try_parse() {
		Ok(args) => args,
		Err(e) => {
			eprintln!("{e}");
			return Ok(());
		}
	};

	region.reset();

	let raw_data = fs::read_to_string(file)?;

	let filtered_data = raw_data
		.chars()
		.filter(|c| matches!(c, '+'..='.' | '>' | '<' | '[' | ']'))
		.collect::<String>();

	debug_span!("after_read_and_filter").in_scope(|| report_alloc_stats(&mut region));

	let program = {
		let unoptimized = BfParser::new(&filtered_data)
			.scan()?
			.into_iter()
			.collect::<Program>();

		debug_span!("after_parse").in_scope(|| report_alloc_stats(&mut region));

		info!(
			"size of raw: {} bytes (len: {})",
			unoptimized.heap_size(),
			unoptimized.len()
		);
		if optimize {
			region.reset();

			let mut optimizer = Optimizer::new(
				unoptimized,
				OutputMetadataStore::new(HashMetadataStore::new(), PathBuf::new().join("./out"))?,
			);

			let out = optimizer.optimize()?;

			debug_span!("after_optimize").in_scope(|| report_alloc_stats(&mut region));

			out
		} else {
			unoptimized
		}
	};

	write_binary(&program)?;

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

	let output = CopyWriter::new(stdout(), Vec::<u8>::with_capacity(program.min_outputs()));

	region.reset();

	let (profiler, output) = match (program.needs_input(), tape) {
		(true, TapeType::Ptr) => {
			let mut vm = Interpreter::<PtrTape, _, _>::with_profiler(program, stdin(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(true, TapeType::Box) => {
			let mut vm = Interpreter::<BoxTape, _, _>::with_profiler(program, stdin(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(true, TapeType::Vec) => {
			let mut vm = Interpreter::<VecTape, _, _>::with_profiler(program, stdin(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(true, TapeType::Stack) => {
			let mut vm = Interpreter::<StackTape, _, _>::with_profiler(program, stdin(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(false, TapeType::Ptr) => {
			let mut vm = Interpreter::<PtrTape, _, _>::with_profiler(program, empty(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(false, TapeType::Box) => {
			let mut vm = Interpreter::<BoxTape, _, _>::with_profiler(program, empty(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(false, TapeType::Vec) => {
			let mut vm = Interpreter::<VecTape, _, _>::with_profiler(program, empty(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
		(false, TapeType::Stack) => {
			let mut vm = Interpreter::<StackTape, _, _>::with_profiler(program, empty(), output);

			vm.run()?;

			(vm.profiler(), get_interpreter_output(&vm))
		}
	};

	if !matches!(output.last(), Some(b'\n')) {
		println!();
	}

	debug_span!("after_run").in_scope(|| report_alloc_stats(&mut region));

	write_profiler(profiler)?;

	debug_span!("total").in_scope(|| report_alloc_stats(&mut total));

	Ok(())
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
	debug!("allocation stats");

	let stats = region.stat_diff();

	debug!("allocations: {}", stats.allocations);
	debug!("deallocations: {}", stats.deallocations);
	debug!("reallocations: {}", stats.reallocations);
	debug!("bytes allocated: {}", stats.bytes_allocated);
	debug!("bytes deallocated: {}", stats.bytes_deallocated);
	debug!("bytes reallocated: {}", stats.bytes_reallocated);

	region.reset();
}

fn get_interpreter_output<T: Tape, R>(
	interpreter: &Interpreter<T, R, CopyWriter<Stdout, Vec<u8>>>,
) -> Vec<u8> {
	let mut out = interpreter.output().as_ref().into_inner().1.clone();

	out.shrink_to_fit();

	out
}

fn write_binary(program: &Program) -> Result<()> {
	let file = fs::OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open("./out/program.bin")?;

	to_writer_with_config(program, file, Config::new(true, false, 0))?;

	Ok(())
}

fn write_format() -> Result<()> {
	let mut tracer = Tracer::new(TracerConfig::default().default_u8_value(1));

	tracer.trace_simple_type::<BlockInstruction>().unwrap();

	tracer.trace_simple_type::<ScaleAnd>().unwrap();

	tracer.trace_simple_type::<SuperInstruction>().unwrap();

	tracer.trace_simple_type::<Instruction>().unwrap();

	tracer.trace_simple_type::<Program>().unwrap();

	let registry = tracer.registry().unwrap();

	let file = fs::OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open("./out/format.json")?;

	serde_json::to_writer_pretty(file, &registry)?;
	Ok(())
}
