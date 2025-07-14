use std::{fs, path::PathBuf};

use anyhow::{Context as _, Result};
use clap::Parser;
use cranelift::{
	codegen::{
		Context,
		control::ControlPlane,
		ir::{Function, UserFuncName},
		verifier::verify_function,
	},
	prelude::*,
};
use logos::Logos;
use target_lexicon::Triple;

fn main() -> Result<()> {
	_ = fs::remove_dir_all("../../out");

	fs::create_dir_all("../../out")?;

	let args = match Args::try_parse() {
		Ok(x) => x,
		Err(e) => {
			eprintln!("{e}");
			return Ok(());
		}
	};

	let raw_data = fs::read_to_string(args.file_path)?;

	let parsed_opcodes = OpCode::lexer(&raw_data)
		.into_iter()
		.filter_map(Result::ok)
		.collect::<Vec<_>>();

	generate_ir(parsed_opcodes)?;

	Ok(())
}

fn generate_ir(parsed: Vec<OpCode>) -> Result<()> {
	let current_triple = Triple::host();

	let mut settings_builder = settings::builder();
	settings_builder.set("opt_level", "speed")?;

	let isa = isa::lookup(current_triple)?.finish(settings::Flags::new(settings_builder))?;

	let ptr_type = isa.pointer_type();

	let mut sig = Signature::new(isa.default_call_conv());
	sig.returns.push(AbiParam::new(ptr_type));
	sig.params.push(AbiParam::new(ptr_type));

	let mut fn_builder_ctx = FunctionBuilderContext::new();
	let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);

	let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

	let ptr = Variable::new(0);
	builder.declare_var(ptr, ptr_type);

	let exit_block = builder.create_block();
	builder.append_block_param(exit_block, ptr_type);

	let block = {
		let block = builder.create_block();
		builder.seal_block(block);
		builder.append_block_params_for_function_params(block);
		builder.switch_to_block(block);
		block
	};

	let zero = builder.ins().iconst(ptr_type, 0);
	let memory_address = builder.block_params(block)[0];

	let mut stack = Vec::new();
	let mem_flags = MemFlags::new();

	let (write_sig, write_addr) = {
		let mut write_sig = Signature::new(isa.default_call_conv());
		write_sig.params.push(AbiParam::new(types::I8));
		write_sig.returns.push(AbiParam::new(ptr_type));
		let write_sig = builder.import_signature(write_sig);

		let write_addr = std::ptr::null::<()>() as i64;
		let write_addr = builder.ins().iconst(ptr_type, write_addr);
		(write_sig, write_addr)
	};

	let (read_sig, read_addr) = {
		let mut read_sig = Signature::new(isa.default_call_conv());
		read_sig.params.push(AbiParam::new(ptr_type));
		read_sig.returns.push(AbiParam::new(ptr_type));
		let read_sig = builder.import_signature(read_sig);

		let read_addr = std::ptr::null::<()>() as i64;
		let read_addr = builder.ins().iconst(ptr_type, read_addr);
		(read_sig, read_addr)
	};

	for op in parsed {
		match op {
			OpCode::Clear => {
				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);
				builder.ins().store(mem_flags, zero, cell_addr, 0);
			}
			OpCode::Increment => {
				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);
				let cell_value = builder.ins().load(types::I8, mem_flags, cell_addr, 0);
				let cell_value = builder.ins().iadd_imm(cell_value, 1);
				builder.ins().store(mem_flags, cell_value, cell_addr, 0);
			}
			OpCode::Decrement => {
				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);
				let cell_value = builder.ins().load(types::I8, mem_flags, cell_addr, 0);
				let cell_value = builder.ins().iadd_imm(cell_value, -1);
				builder.ins().store(mem_flags, cell_value, cell_addr, 0);
			}
			OpCode::MoveLeft => {
				let ptr_value = builder.use_var(ptr);
				let ptr_minus_one = builder.ins().iadd_imm(ptr_value, -1);

				let wrapped = builder.ins().iconst(ptr_type, 30_000 - 1);
				let ptr_value = builder.ins().select(ptr_value, ptr_minus_one, wrapped);

				builder.def_var(ptr, ptr_value);
			}
			OpCode::MoveRight => {
				let ptr_value = builder.use_var(ptr);
				let ptr_plus_one = builder.ins().iadd_imm(ptr_value, 1);

				let cmp = builder.ins().icmp_imm(IntCC::Equal, ptr_plus_one, 30_000);
				let ptr_value = builder.ins().select(cmp, zero, ptr_plus_one);

				builder.def_var(ptr, ptr_value);
			}
			OpCode::JumpRight => {
				let inner_block = builder.create_block();
				let after_block = builder.create_block();

				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);
				let cell_value = builder.ins().load(types::I8, mem_flags, cell_addr, 0);

				builder
					.ins()
					.brif(cell_value, inner_block, &[], after_block, &[]);

				builder.switch_to_block(inner_block);

				stack.push((inner_block, after_block));
			}
			OpCode::JumpLeft => {
				let (inner_block, after_block) = stack.pop().context("unmatched brackets")?;
				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);
				let cell_value = builder.ins().load(types::I8, mem_flags, cell_addr, 0);

				builder
					.ins()
					.brif(cell_value, inner_block, &[], after_block, &[]);

				builder.seal_block(inner_block);
				builder.seal_block(after_block);

				builder.switch_to_block(after_block);
			}
			OpCode::Output => {
				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);
				let cell_value = builder.ins().load(types::I8, mem_flags, cell_addr, 0);

				let inst = builder
					.ins()
					.call_indirect(write_sig, write_addr, &[cell_value]);
				let result = builder.inst_results(inst)[0];

				let after_block = builder.create_block();

				builder
					.ins()
					.brif(result, exit_block, &[result.into()], after_block, &[]);

				builder.seal_block(after_block);
				builder.switch_to_block(after_block);
			}
			OpCode::Input => {
				let ptr_value = builder.use_var(ptr);
				let cell_addr = builder.ins().iadd(memory_address, ptr_value);

				let inst = builder
					.ins()
					.call_indirect(read_sig, read_addr, &[cell_addr]);
				let result = builder.inst_results(inst)[0];

				let after_block = builder.create_block();

				builder
					.ins()
					.brif(result, exit_block, &[result.into()], after_block, &[]);

				builder.seal_block(after_block);
				builder.switch_to_block(after_block);
			}
		}
	}

	{
		builder.ins().return_(&[zero]);

		builder.switch_to_block(exit_block);
		builder.seal_block(exit_block);

		let result = builder.block_params(exit_block)[0];
		builder.ins().return_(&[result]);

		builder.seal_all_blocks();

		builder.finalize();
	}

	verify_function(&func, &*isa)?;
	fs::write("../../out/unoptimized.ir", func.display().to_string())?;

	let (optimized, code) = {
		let mut ctx = Context::for_function(func);
		let mut plane = ControlPlane::default();

		ctx.optimize(&*isa, &mut plane)?;

		(
			ctx.func.clone(),
			ctx.compile(&*isa, &mut plane).unwrap().clone(),
		)
	};

	let buffer = code.code_buffer();

	fs::write("../../out/program.bin", buffer)?;
	fs::write("../../out/optimized.ir", optimized.display().to_string())?;

	Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
enum OpCode {
	#[token("<")]
	MoveLeft,
	#[token(">")]
	MoveRight,
	#[token("+")]
	Increment,
	#[token("-")]
	Decrement,
	#[token(",")]
	Input,
	#[token(".")]
	Output,
	#[token("]")]
	JumpLeft,
	#[token("[")]
	JumpRight,
	#[token("[-]")]
	Clear,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
	pub file_path: PathBuf,
}
