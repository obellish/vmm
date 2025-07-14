#![allow(clippy::mut_mut)]

use anyhow::{Context as _, Result};
use koopa::ir::{
	BasicBlock, BinaryOp, Function, FunctionData, Program, Type, Value,
	builder::{BasicBlockBuilder, GlobalInstBuilder, LocalInstBuilder, ValueBuilder},
};

struct Environment<'a> {
	ptr: Value,
	putchar: Function,
	getchar: Function,
	main: &'a mut FunctionData,
}

macro_rules! bb {
	($func:expr) => {
		$func.dfg_mut().new_bb()
	};
	($func:expr, $bb:expr) => {
		$func
			.layout_mut()
			.bbs_mut()
			.push_key_back($bb)
			.ok()
			.context("failed to add basic_block")
	};
}

macro_rules! new_value {
	($func:expr) => {
		$func.dfg_mut().new_value()
	};
}

macro_rules! inst {
	($func:expr, $bb:expr, $inst:expr) => {
		$func
			.layout_mut()
			.bb_mut($bb)
			.insts_mut()
			.push_key_back($inst)
			.ok()
			.context("failed to add instruction")
	};
}

pub fn parse_program(input: Vec<u8>) -> Result<Program> {
	let mut program = Program::new();
	let zero = program
		.new_value()
		.zero_init(Type::get_array(Type::get_i32(), 65536));
	let ptr = program.new_value().global_alloc(zero);
	program.set_value_name(ptr, Some("@data_arr".to_owned()));

	let putchar = FunctionData::new_decl(
		"@putchar".to_owned(),
		vec![Type::get_i32()],
		Type::get_i32(),
	);
	let putchar = program.new_func(putchar);
	let getchar = FunctionData::new_decl("@getchar".to_owned(), Vec::new(), Type::get_i32());
	let getchar = program.new_func(getchar);
	let main = FunctionData::new("@main".to_owned(), Vec::new(), Type::get_i32());
	let main = program.new_func(main);

	generate_main(
		&input,
		Environment {
			ptr,
			putchar,
			getchar,
			main: program.func_mut(main),
		},
	)?;

	Ok(program)
}

fn generate_main(input: &[u8], mut env: Environment<'_>) -> Result<()> {
	let main = &mut env.main;
	let entry = bb!(main).basic_block(Some("%entry".to_owned()));
	bb!(main, entry)?;
	let ptr = new_value!(main).alloc(Type::get_pointer(Type::get_i32()));
	main.dfg_mut().set_value_name(ptr, Some("%ptr".to_owned()));
	inst!(main, entry, ptr)?;
	let zero = new_value!(main).integer(0);
	let data_ptr = new_value!(main).get_elem_ptr(env.ptr, zero);
	inst!(main, entry, data_ptr)?;
	let store = new_value!(main).store(data_ptr, ptr);
	inst!(main, entry, store)?;
	env.ptr = ptr;

	let bb = generate_bbs(input, &mut env, entry)?;

	let main = &mut env.main;
	let end = bb!(main).basic_block(Some("%end".to_owned()));
	bb!(main, end)?;
	let jump = new_value!(main).jump(end);
	inst!(main, bb, jump)?;
	let ret = new_value!(main).ret(Some(zero));
	inst!(main, end, ret)?;

	Ok(())
}

fn generate_bbs(input: &[u8], env: &mut Environment<'_>, entry: BasicBlock) -> Result<BasicBlock> {
	let mut bb = bb!(env.main).basic_block(None);
	bb!(env.main, bb)?;
	let jump = new_value!(env.main).jump(bb);
	inst!(env.main, entry, jump)?;
	let mut loop_info = Vec::new();

	for byte in input {
		bb = match *byte {
			b'>' => generate_ptr_op(env, bb, 1),
			b'<' => generate_ptr_op(env, bb, -1),
			b'+' => generate_data_op(env, bb, 1),
			b'-' => generate_data_op(env, bb, -1),
			b'[' => generate_start(env, bb, &mut loop_info),
			b']' => generate_end(env, bb, &mut loop_info),
			b'.' => generate_put(env, bb),
			b',' => generate_get(env, bb),
			_ => continue,
		}?;
	}

	Ok(bb)
}

fn generate_ptr_op(env: &mut Environment<'_>, bb: BasicBlock, i: i32) -> Result<BasicBlock> {
	let main = &mut env.main;
	let load = new_value!(main).load(env.ptr);
	inst!(main, bb, load)?;
	let index = new_value!(main).integer(i);
	let gp = new_value!(main).get_ptr(load, index);
	inst!(main, bb, gp)?;
	let store = new_value!(main).store(gp, env.ptr);
	inst!(main, bb, store)?;

	Ok(bb)
}

fn generate_data_op(env: &mut Environment<'_>, bb: BasicBlock, i: i32) -> Result<BasicBlock> {
	let main = &mut env.main;
	let load = new_value!(main).load(env.ptr);
	inst!(main, bb, load)?;
	let data = new_value!(main).load(load);
	inst!(main, bb, data)?;
	let rhs = new_value!(main).integer(i);
	let add = new_value!(main).binary(BinaryOp::Add, data, rhs);
	inst!(main, bb, add)?;
	let store = new_value!(main).store(add, load);
	inst!(main, bb, store)?;

	Ok(bb)
}

fn generate_start(
	env: &mut Environment<'_>,
	bb: BasicBlock,
	loop_info: &mut Vec<(BasicBlock, BasicBlock)>,
) -> Result<BasicBlock> {
	let main = &mut env.main;
	let cond_bb = bb!(main).basic_block(Some("%while_cond".to_owned()));
	bb!(main, cond_bb)?;

	let jump = new_value!(main).jump(cond_bb);
	inst!(main, bb, jump)?;

	let load = new_value!(main).load(env.ptr);
	inst!(main, cond_bb, load)?;
	let data = new_value!(main).load(load);
	inst!(main, cond_bb, data)?;
	let zero = new_value!(main).integer(0);
	let cmp = new_value!(main).binary(BinaryOp::NotEq, data, zero);
	inst!(main, cond_bb, cmp)?;
	let body_bb = bb!(main).basic_block(Some("%while_body".to_owned()));
	let end_bb = bb!(main).basic_block(Some("%while_end".to_owned()));
	let br = new_value!(main).branch(cmp, body_bb, end_bb);
	inst!(main, cond_bb, br)?;
	bb!(main, body_bb)?;

	loop_info.push((cond_bb, end_bb));

	Ok(body_bb)
}

fn generate_end(
	env: &mut Environment<'_>,
	bb: BasicBlock,
	loop_info: &mut Vec<(BasicBlock, BasicBlock)>,
) -> Result<BasicBlock> {
	let (cond_bb, end_bb) = loop_info
		.pop()
		.context("parsing error: mismatched brackets")?;
	let jump = new_value!(env.main).jump(cond_bb);
	inst!(env.main, bb, jump)?;
	bb!(env.main, end_bb)?;
	Ok(end_bb)
}

fn generate_put(env: &mut Environment<'_>, bb: BasicBlock) -> Result<BasicBlock> {
	let main = &mut env.main;
	let load = new_value!(main).load(env.ptr);
	inst!(main, bb, load)?;
	let data = new_value!(main).load(load);
	inst!(main, bb, data)?;
	let call = new_value!(main).call(env.putchar, vec![data]);
	inst!(main, bb, call)?;

	Ok(bb)
}

fn generate_get(env: &mut Environment<'_>, bb: BasicBlock) -> Result<BasicBlock> {
	let main = &mut env.main;
	let call = new_value!(main).call(env.getchar, Vec::new());
	inst!(main, bb, call)?;
	let load = new_value!(main).load(env.ptr);
	inst!(main, bb, load)?;
	let store = new_value!(main).store(call, load);
	inst!(main, bb, store)?;
	Ok(bb)
}
