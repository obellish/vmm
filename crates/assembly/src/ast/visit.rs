#![allow(clippy::trivially_copy_pass_by_ref)]

use core::ops::ControlFlow;

use super::{
	AliasTarget, Block, DebugOptions, ErrorCode, Export, ImmFelt, ImmU8, ImmU16, ImmU32, Import,
	Instruction, InvocationTarget, Module, Op, Procedure, ProcedureAlias, SystemEventNode,
};
use crate::Span;

pub trait Visit<T = ()> {
	fn visit_module(&mut self, module: &Module) -> ControlFlow<T> {
		visit_module(self, module)
	}

	fn visit_import(&mut self, import: &Import) -> ControlFlow<T> {
		visit_import(self, import)
	}

	fn visit_export(&mut self, export: &Export) -> ControlFlow<T> {
		visit_export(self, export)
	}

	fn visit_procedure(&mut self, proc: &Procedure) -> ControlFlow<T> {
		visit_procedure(self, proc)
	}

	fn visit_procedure_alias(&mut self, alias: &ProcedureAlias) -> ControlFlow<T> {
		visit_procedure_alias(self, alias)
	}

	fn visit_block(&mut self, block: &Block) -> ControlFlow<T> {
		visit_block(self, block)
	}

	fn visit_op(&mut self, op: &Op) -> ControlFlow<T> {
		visit_op(self, op)
	}

	fn visit_inst(&mut self, inst: &Span<Instruction>) -> ControlFlow<T> {
		visit_inst(self, inst)
	}

	fn visit_system_event(&mut self, sys_event: Span<&SystemEventNode>) -> ControlFlow<T> {
		visit_system_event(self, sys_event)
	}

	fn visit_debug_options(&mut self, options: Span<&DebugOptions>) -> ControlFlow<T> {
		visit_debug_options(self, options)
	}

	fn visit_exec(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		visit_exec(self, target)
	}

	fn visit_call(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		visit_call(self, target)
	}

	fn visit_syscall(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		visit_syscall(self, target)
	}

	fn visit_procref(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		visit_procref(self, target)
	}

	fn visit_invoke_target(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		visit_invoke_target(self, target)
	}

	fn visit_alias_target(&mut self, target: &AliasTarget) -> ControlFlow<T> {
		visit_alias_target(self, target)
	}

	fn visit_immediate_u8(&mut self, imm: &ImmU8) -> ControlFlow<T> {
		visit_immediate_u8(self, imm)
	}

	fn visit_immediate_u16(&mut self, imm: &ImmU16) -> ControlFlow<T> {
		visit_immediate_u16(self, imm)
	}

	fn visit_immediate_u32(&mut self, imm: &ImmU32) -> ControlFlow<T> {
		visit_immediate_u32(self, imm)
	}

	fn visit_immediate_felt(&mut self, imm: &ImmFelt) -> ControlFlow<T> {
		visit_immediate_felt(self, imm)
	}

	fn visit_immediate_error_code(&mut self, code: &ErrorCode) -> ControlFlow<T> {
		visit_immediate_error_code(self, code)
	}
}

impl<V, T> Visit<T> for &mut V
where
	V: ?Sized + Visit<T>,
{
	fn visit_module(&mut self, module: &Module) -> ControlFlow<T> {
		(**self).visit_module(module)
	}

	fn visit_import(&mut self, import: &Import) -> ControlFlow<T> {
		(**self).visit_import(import)
	}

	fn visit_export(&mut self, export: &Export) -> ControlFlow<T> {
		(**self).visit_export(export)
	}

	fn visit_procedure(&mut self, procedure: &Procedure) -> ControlFlow<T> {
		(**self).visit_procedure(procedure)
	}

	fn visit_procedure_alias(&mut self, alias: &ProcedureAlias) -> ControlFlow<T> {
		(**self).visit_procedure_alias(alias)
	}

	fn visit_block(&mut self, block: &Block) -> ControlFlow<T> {
		(**self).visit_block(block)
	}

	fn visit_op(&mut self, op: &Op) -> ControlFlow<T> {
		(**self).visit_op(op)
	}

	fn visit_inst(&mut self, inst: &Span<Instruction>) -> ControlFlow<T> {
		(**self).visit_inst(inst)
	}

	fn visit_system_event(&mut self, sys_event: Span<&SystemEventNode>) -> ControlFlow<T> {
		(**self).visit_system_event(sys_event)
	}

	fn visit_debug_options(&mut self, options: Span<&DebugOptions>) -> ControlFlow<T> {
		(**self).visit_debug_options(options)
	}

	fn visit_exec(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		(**self).visit_exec(target)
	}

	fn visit_call(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		(**self).visit_call(target)
	}

	fn visit_syscall(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		(**self).visit_syscall(target)
	}

	fn visit_procref(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		(**self).visit_procref(target)
	}

	fn visit_invoke_target(&mut self, target: &InvocationTarget) -> ControlFlow<T> {
		(**self).visit_invoke_target(target)
	}

	fn visit_alias_target(&mut self, target: &AliasTarget) -> ControlFlow<T> {
		(**self).visit_alias_target(target)
	}

	fn visit_immediate_u8(&mut self, imm: &ImmU8) -> ControlFlow<T> {
		(**self).visit_immediate_u8(imm)
	}

	fn visit_immediate_u16(&mut self, imm: &ImmU16) -> ControlFlow<T> {
		(**self).visit_immediate_u16(imm)
	}

	fn visit_immediate_u32(&mut self, imm: &ImmU32) -> ControlFlow<T> {
		(**self).visit_immediate_u32(imm)
	}

	fn visit_immediate_felt(&mut self, imm: &ImmFelt) -> ControlFlow<T> {
		(**self).visit_immediate_felt(imm)
	}

	fn visit_immediate_error_code(&mut self, code: &ErrorCode) -> ControlFlow<T> {
		(**self).visit_immediate_error_code(code)
	}
}

pub trait VisitMut<T = ()> {
	fn visit_module_mut(&mut self, module: &mut Module) -> ControlFlow<T> {
		visit_module_mut(self, module)
	}

	fn visit_import_mut(&mut self, import: &mut Import) -> ControlFlow<T> {
		visit_import_mut(self, import)
	}

	fn visit_export_mut(&mut self, export: &mut Export) -> ControlFlow<T> {
		visit_export_mut(self, export)
	}

	fn visit_procedure_mut(&mut self, proc: &mut Procedure) -> ControlFlow<T> {
		visit_procedure_mut(self, proc)
	}

	fn visit_procedure_alias_mut(&mut self, alias: &mut ProcedureAlias) -> ControlFlow<T> {
		visit_procedure_alias_mut(self, alias)
	}

	fn visit_block_mut(&mut self, block: &mut Block) -> ControlFlow<T> {
		visit_block_mut(self, block)
	}

	fn visit_op_mut(&mut self, op: &mut Op) -> ControlFlow<T> {
		visit_op_mut(self, op)
	}

	fn visit_inst_mut(&mut self, inst: &mut Span<Instruction>) -> ControlFlow<T> {
		visit_inst_mut(self, inst)
	}

	fn visit_system_event_mut(&mut self, sys_event: Span<&mut SystemEventNode>) -> ControlFlow<T> {
		visit_system_event_mut(self, sys_event)
	}

	fn visit_debug_options_mut(&mut self, options: Span<&mut DebugOptions>) -> ControlFlow<T> {
		visit_debug_options_mut(self, options)
	}

	fn visit_exec_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		visit_exec_mut(self, target)
	}

	fn visit_call_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		visit_call_mut(self, target)
	}

	fn visit_syscall_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		visit_syscall_mut(self, target)
	}

	fn visit_procref_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		visit_procref_mut(self, target)
	}

	fn visit_invoke_target_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		visit_invoke_target_mut(self, target)
	}

	fn visit_alias_target_mut(&mut self, target: &mut AliasTarget) -> ControlFlow<T> {
		visit_alias_target_mut(self, target)
	}

	fn visit_immediate_u8_mut(&mut self, imm: &mut ImmU8) -> ControlFlow<T> {
		visit_immediate_u8_mut(self, imm)
	}

	fn visit_immediate_u16_mut(&mut self, imm: &mut ImmU16) -> ControlFlow<T> {
		visit_immediate_u16_mut(self, imm)
	}

	fn visit_immediate_u32_mut(&mut self, imm: &mut ImmU32) -> ControlFlow<T> {
		visit_immediate_u32_mut(self, imm)
	}

	fn visit_immediate_felt_mut(&mut self, imm: &mut ImmFelt) -> ControlFlow<T> {
		visit_immediate_felt_mut(self, imm)
	}

	fn visit_immediate_error_code_mut(&mut self, code: &mut ErrorCode) -> ControlFlow<T> {
		visit_immediate_error_code_mut(self, code)
	}
}

impl<V, T> VisitMut<T> for &mut V
where
	V: ?Sized + VisitMut<T>,
{
	fn visit_module_mut(&mut self, module: &mut Module) -> ControlFlow<T> {
		(**self).visit_module_mut(module)
	}

	fn visit_import_mut(&mut self, import: &mut Import) -> ControlFlow<T> {
		(**self).visit_import_mut(import)
	}

	fn visit_export_mut(&mut self, export: &mut Export) -> ControlFlow<T> {
		(**self).visit_export_mut(export)
	}

	fn visit_procedure_mut(&mut self, proc: &mut Procedure) -> ControlFlow<T> {
		(**self).visit_procedure_mut(proc)
	}

	fn visit_procedure_alias_mut(&mut self, alias: &mut ProcedureAlias) -> ControlFlow<T> {
		(**self).visit_procedure_alias_mut(alias)
	}

	fn visit_block_mut(&mut self, block: &mut Block) -> ControlFlow<T> {
		(**self).visit_block_mut(block)
	}

	fn visit_op_mut(&mut self, op: &mut Op) -> ControlFlow<T> {
		(**self).visit_op_mut(op)
	}

	fn visit_inst_mut(&mut self, inst: &mut Span<Instruction>) -> ControlFlow<T> {
		(**self).visit_inst_mut(inst)
	}

	fn visit_system_event_mut(&mut self, sys_event: Span<&mut SystemEventNode>) -> ControlFlow<T> {
		(**self).visit_system_event_mut(sys_event)
	}

	fn visit_debug_options_mut(&mut self, options: Span<&mut DebugOptions>) -> ControlFlow<T> {
		(**self).visit_debug_options_mut(options)
	}

	fn visit_exec_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		(**self).visit_exec_mut(target)
	}

	fn visit_call_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		(**self).visit_call_mut(target)
	}

	fn visit_syscall_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		(**self).visit_syscall_mut(target)
	}

	fn visit_procref_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		(**self).visit_procref_mut(target)
	}

	fn visit_invoke_target_mut(&mut self, target: &mut InvocationTarget) -> ControlFlow<T> {
		(**self).visit_invoke_target_mut(target)
	}

	fn visit_alias_target_mut(&mut self, target: &mut AliasTarget) -> ControlFlow<T> {
		(**self).visit_alias_target_mut(target)
	}

	fn visit_immediate_u8_mut(&mut self, imm: &mut ImmU8) -> ControlFlow<T> {
		(**self).visit_immediate_u8_mut(imm)
	}

	fn visit_immediate_u16_mut(&mut self, imm: &mut ImmU16) -> ControlFlow<T> {
		(**self).visit_immediate_u16_mut(imm)
	}

	fn visit_immediate_u32_mut(&mut self, imm: &mut ImmU32) -> ControlFlow<T> {
		(**self).visit_immediate_u32_mut(imm)
	}

	fn visit_immediate_felt_mut(&mut self, imm: &mut ImmFelt) -> ControlFlow<T> {
		(**self).visit_immediate_felt_mut(imm)
	}

	fn visit_immediate_error_code_mut(&mut self, code: &mut ErrorCode) -> ControlFlow<T> {
		(**self).visit_immediate_error_code_mut(code)
	}
}

pub fn visit_module<V, T>(visitor: &mut V, module: &Module) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	for import in module.imports() {
		visitor.visit_import(import)?;
	}

	for export in module.procedures() {
		visitor.visit_export(export)?;
	}

	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_import<V, T>(_: &mut V, _: &Import) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

pub fn visit_export<V, T>(visitor: &mut V, export: &Export) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	match export {
		Export::Procedure(proc) => visitor.visit_procedure(proc),
		Export::Alias(alias) => visitor.visit_procedure_alias(alias),
	}
}

pub fn visit_procedure<V, T>(visitor: &mut V, procedure: &Procedure) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	visitor.visit_block(procedure.body())
}

pub fn visit_procedure_alias<V, T>(visitor: &mut V, alias: &ProcedureAlias) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	visitor.visit_alias_target(alias.target())
}

pub fn visit_block<V, T>(visitor: &mut V, block: &Block) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	for op in block {
		visitor.visit_op(op)?;
	}

	ControlFlow::Continue(())
}

pub fn visit_op<V, T>(visitor: &mut V, op: &Op) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	match op {
		Op::If {
			then_blk, else_blk, ..
		} => {
			visitor.visit_block(then_blk)?;
			visitor.visit_block(else_blk)
		}
		Op::While { body, .. } | Op::Repeat { body, .. } => visitor.visit_block(body),
		Op::Inst(inst) => visitor.visit_inst(inst),
	}
}

pub fn visit_inst<V, T>(visitor: &mut V, inst: &Span<Instruction>) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	let span = inst.span();
	match &**inst {
		Instruction::U32ShrImm(imm)
		| Instruction::U32ShlImm(imm)
		| Instruction::U32RotrImm(imm)
		| Instruction::U32RotlImm(imm)
		| Instruction::AdvPush(imm) => visitor.visit_immediate_u8(imm),
		Instruction::AssertWithError(code)
		| Instruction::AssertEqWithError(code)
		| Instruction::AssertEqwWithError(code)
		| Instruction::AssertzWithError(code)
		| Instruction::U32AssertWithError(code)
		| Instruction::U32Assert2WithError(code)
		| Instruction::U32AssertWWithError(code)
		| Instruction::MTreeVerifyWithError(code) => visitor.visit_immediate_error_code(code),
		Instruction::AddImm(imm)
		| Instruction::SubImm(imm)
		| Instruction::MulImm(imm)
		| Instruction::DivImm(imm)
		| Instruction::ExpImm(imm)
		| Instruction::EqImm(imm)
		| Instruction::NeqImm(imm)
		| Instruction::Push(imm) => visitor.visit_immediate_felt(imm),
		Instruction::U32WrappingAddImm(imm)
		| Instruction::U32OverflowingAddImm(imm)
		| Instruction::U32WrappingSubImm(imm)
		| Instruction::U32OverflowingSubImm(imm)
		| Instruction::U32WrappingMulImm(imm)
		| Instruction::U32OverflowingMulImm(imm)
		| Instruction::U32DivImm(imm)
		| Instruction::U32ModImm(imm)
		| Instruction::U32DivModImm(imm)
		| Instruction::MemLoadImm(imm)
		| Instruction::MemLoadWImm(imm)
		| Instruction::MemStoreImm(imm)
		| Instruction::MemStoreWImm(imm)
		| Instruction::Emit(imm)
		| Instruction::Trace(imm) => visitor.visit_immediate_u32(imm),
		Instruction::SysEvent(sys_event) => visitor.visit_system_event(Span::new(span, sys_event)),
		Instruction::Exec(target) => visitor.visit_exec(target),
		Instruction::Call(target) => visitor.visit_call(target),
		Instruction::SysCall(target) => visitor.visit_syscall(target),
		Instruction::ProcRef(target) => visitor.visit_procref(target),
		Instruction::Debug(options) => visitor.visit_debug_options(Span::new(span, options)),
		_ => ControlFlow::Continue(()),
	}
}

#[must_use]
pub fn visit_system_event<V, T>(_: &mut V, _: Span<&SystemEventNode>) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

pub fn visit_debug_options<V, T>(visitor: &mut V, options: Span<&DebugOptions>) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	match options.into_inner() {
		DebugOptions::StackTop(imm) => visitor.visit_immediate_u8(imm),
		DebugOptions::LocalRangeFrom(imm) => visitor.visit_immediate_u16(imm),
		DebugOptions::MemInterval(imm1, imm2) => {
			visitor.visit_immediate_u32(imm1)?;
			visitor.visit_immediate_u32(imm2)
		}
		DebugOptions::LocalInterval(imm1, imm2) => {
			visitor.visit_immediate_u16(imm1)?;
			visitor.visit_immediate_u16(imm2)
		}
		_ => ControlFlow::Continue(()),
	}
}

pub fn visit_exec<V, T>(visitor: &mut V, target: &InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	visitor.visit_invoke_target(target)
}

pub fn visit_call<V, T>(visitor: &mut V, target: &InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	visitor.visit_invoke_target(target)
}

pub fn visit_syscall<V, T>(visitor: &mut V, target: &InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	visitor.visit_invoke_target(target)
}

pub fn visit_procref<V, T>(visitor: &mut V, target: &InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	visitor.visit_invoke_target(target)
}

#[must_use]
pub fn visit_invoke_target<V, T>(_: &mut V, _: &InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_alias_target<V, T>(_: &mut V, _: &AliasTarget) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_u8<V, T>(_: &mut V, _: &ImmU8) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_u16<V, T>(_: &mut V, _: &ImmU16) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_u32<V, T>(_: &mut V, _: &ImmU32) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_felt<V, T>(_: &mut V, _: &ImmFelt) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_error_code<V, T>(_: &mut V, _: &ErrorCode) -> ControlFlow<T>
where
	V: ?Sized + Visit<T>,
{
	ControlFlow::Continue(())
}

pub fn visit_module_mut<V, T>(visitor: &mut V, module: &mut Module) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	for import in module.imports_mut() {
		visitor.visit_import_mut(import)?;
	}

	for export in module.procedures_mut() {
		visitor.visit_export_mut(export)?;
	}

	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_import_mut<V, T>(_: &mut V, _: &mut Import) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

pub fn visit_export_mut<V, T>(visitor: &mut V, export: &mut Export) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	match export {
		Export::Procedure(proc) => visitor.visit_procedure_mut(proc),
		Export::Alias(alias) => visitor.visit_procedure_alias_mut(alias),
	}
}

pub fn visit_procedure_mut<V, T>(visitor: &mut V, procedure: &mut Procedure) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	visitor.visit_block_mut(procedure.body_mut())
}

pub fn visit_procedure_alias_mut<V, T>(
	visitor: &mut V,
	alias: &mut ProcedureAlias,
) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	visitor.visit_alias_target_mut(alias.target_mut())
}

pub fn visit_block_mut<V, T>(visitor: &mut V, block: &mut Block) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	for op in block.iter_mut() {
		visitor.visit_op_mut(op)?;
	}

	ControlFlow::Continue(())
}

pub fn visit_op_mut<V, T>(visitor: &mut V, op: &mut Op) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	match op {
		Op::If {
			then_blk, else_blk, ..
		} => {
			visitor.visit_block_mut(then_blk)?;
			visitor.visit_block_mut(else_blk)
		}
		Op::While { body, .. } | Op::Repeat { body, .. } => visitor.visit_block_mut(body),
		Op::Inst(inst) => visitor.visit_inst_mut(inst),
	}
}

pub fn visit_inst_mut<V, T>(visitor: &mut V, inst: &mut Span<Instruction>) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	let span = inst.span();
	match &mut **inst {
		Instruction::U32ShlImm(imm)
		| Instruction::U32ShrImm(imm)
		| Instruction::U32RotrImm(imm)
		| Instruction::U32RotlImm(imm)
		| Instruction::AdvPush(imm) => visitor.visit_immediate_u8_mut(imm),
		Instruction::Locaddr(imm)
		| Instruction::LocLoad(imm)
		| Instruction::LocLoadW(imm)
		| Instruction::LocStore(imm)
		| Instruction::LocStoreW(imm) => visitor.visit_immediate_u16_mut(imm),
		Instruction::AssertWithError(code)
		| Instruction::AssertEqWithError(code)
		| Instruction::AssertEqwWithError(code)
		| Instruction::AssertzWithError(code)
		| Instruction::U32AssertWithError(code)
		| Instruction::U32Assert2WithError(code)
		| Instruction::U32AssertWWithError(code)
		| Instruction::MTreeVerifyWithError(code) => visitor.visit_immediate_error_code_mut(code),
		Instruction::AddImm(imm)
		| Instruction::SubImm(imm)
		| Instruction::MulImm(imm)
		| Instruction::DivImm(imm)
		| Instruction::ExpImm(imm)
		| Instruction::EqImm(imm)
		| Instruction::NeqImm(imm)
		| Instruction::Push(imm) => visitor.visit_immediate_felt_mut(imm),
		Instruction::U32WrappingAddImm(imm)
		| Instruction::U32OverflowingAddImm(imm)
		| Instruction::U32WrappingSubImm(imm)
		| Instruction::U32OverflowingSubImm(imm)
		| Instruction::U32WrappingMulImm(imm)
		| Instruction::U32OverflowingMulImm(imm)
		| Instruction::U32DivImm(imm)
		| Instruction::U32ModImm(imm)
		| Instruction::U32DivModImm(imm)
		| Instruction::MemLoadImm(imm)
		| Instruction::MemLoadWImm(imm)
		| Instruction::MemStoreImm(imm)
		| Instruction::MemStoreWImm(imm)
		| Instruction::Emit(imm)
		| Instruction::Trace(imm) => visitor.visit_immediate_u32_mut(imm),
		Instruction::SysEvent(sys_event) => {
			visitor.visit_system_event_mut(Span::new(span, sys_event))
		}
		Instruction::Exec(target) => visitor.visit_exec_mut(target),
		Instruction::Call(target) => visitor.visit_call_mut(target),
		Instruction::SysCall(target) => visitor.visit_syscall_mut(target),
		Instruction::ProcRef(target) => visitor.visit_procref_mut(target),
		Instruction::Debug(options) => visitor.visit_debug_options_mut(Span::new(span, options)),
		_ => ControlFlow::Continue(()),
	}
}

#[must_use]
pub fn visit_system_event_mut<V, T>(_: &mut V, _: Span<&mut SystemEventNode>) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

pub fn visit_debug_options_mut<V, T>(
	visitor: &mut V,
	options: Span<&mut DebugOptions>,
) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	match options.into_inner() {
		DebugOptions::StackTop(imm) => visitor.visit_immediate_u8_mut(imm),
		DebugOptions::LocalRangeFrom(imm) => visitor.visit_immediate_u16_mut(imm),
		DebugOptions::MemInterval(imm1, imm2) => {
			visitor.visit_immediate_u32_mut(imm1)?;
			visitor.visit_immediate_u32_mut(imm2)
		}
		DebugOptions::LocalInterval(imm1, imm2) => {
			visitor.visit_immediate_u16_mut(imm1)?;
			visitor.visit_immediate_u16_mut(imm2)
		}
		_ => ControlFlow::Continue(()),
	}
}

pub fn visit_exec_mut<V, T>(visitor: &mut V, target: &mut InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	visitor.visit_invoke_target_mut(target)
}

pub fn visit_call_mut<V, T>(visitor: &mut V, target: &mut InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	visitor.visit_invoke_target_mut(target)
}

pub fn visit_syscall_mut<V, T>(visitor: &mut V, target: &mut InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	visitor.visit_invoke_target_mut(target)
}

pub fn visit_procref_mut<V, T>(visitor: &mut V, target: &mut InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	visitor.visit_invoke_target_mut(target)
}

#[must_use]
pub fn visit_invoke_target_mut<V, T>(_: &mut V, _: &mut InvocationTarget) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_alias_target_mut<V, T>(_: &mut V, _: &mut AliasTarget) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_u8_mut<V, T>(_: &mut V, _: &mut ImmU8) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_u16_mut<V, T>(_: &mut V, _: &mut ImmU16) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_u32_mut<V, T>(_: &mut V, _: &mut ImmU32) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_felt_mut<V, T>(_: &mut V, _: &mut ImmFelt) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}

#[must_use]
pub fn visit_immediate_error_code_mut<V, T>(_: &mut V, _: &mut ErrorCode) -> ControlFlow<T>
where
	V: ?Sized + VisitMut<T>,
{
	ControlFlow::Continue(())
}
