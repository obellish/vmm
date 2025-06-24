#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(portable_simd))]

mod profiler;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind as IoErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
	mem,
	num::NonZeroU8,
};

use vmm_ir::{BlockInstruction, Instruction, Offset, ScaleAnd, SuperInstruction, WriteInstruction};
use vmm_num::ops::{WrappingAddAssign, WrappingMul, WrappingMulAssign, WrappingSubAssign};
use vmm_program::Program;
use vmm_tape::{Cell, Tape, TapePointer};
use vmm_utils::GetOrZero as _;

pub use self::profiler::*;

pub const ITERATION_LIMIT: usize = 100_000;

#[derive(Debug, Clone)]
pub struct Interpreter<T, R = Stdin, W = Stdout> {
	program: Program,
	input: R,
	output: W,
	profiler: Option<Profiler>,
	tape: T,
}

impl<T: Tape, R, W> Interpreter<T, R, W> {
	#[inline]
	pub fn new(program: Program, input: R, output: W) -> Self {
		Self {
			program,
			input,
			output,
			profiler: None,
			tape: {
				let mut t = T::default();

				t.init();

				t
			},
		}
	}

	#[inline]
	pub fn with_profiler(program: Program, input: R, output: W) -> Self {
		Self::new(program, input, output).and_with_profiler()
	}

	#[inline]
	#[must_use]
	pub const fn and_with_profiler(mut self) -> Self {
		self.profiler = Some(Profiler::new());
		self
	}

	#[inline]
	pub fn profiler(&self) -> Profiler {
		self.profiler.unwrap_or_default()
	}

	#[inline]
	pub const fn program(&self) -> &Program {
		&self.program
	}

	#[inline]
	pub const fn program_mut(&mut self) -> &mut Program {
		&mut self.program
	}

	#[inline]
	pub const fn tape(&self) -> &T {
		&self.tape
	}

	#[inline]
	pub const fn tape_mut(&mut self) -> &mut T {
		&mut self.tape
	}

	#[inline]
	pub fn current_cell(&self) -> &Cell {
		unsafe { self.tape().current_cell_unchecked() }
	}

	#[inline]
	pub fn cell_mut(&mut self) -> &mut Cell {
		unsafe { self.tape_mut().current_cell_unchecked_mut() }
	}

	#[inline]
	pub fn ptr(&self) -> &TapePointer {
		self.tape().ptr()
	}

	#[inline]
	pub fn ptr_mut(&mut self) -> &mut TapePointer {
		self.tape_mut().ptr_mut()
	}

	pub const fn input(&self) -> &R {
		&self.input
	}

	pub const fn output(&self) -> &W {
		&self.output
	}
}

#[allow(clippy::unused_self)]
impl<T: Tape, R, W> Interpreter<T, R, W>
where
	R: Read + 'static,
	W: Write + 'static,
{
	#[inline]
	pub fn into_dyn(self) -> Interpreter<T, Box<dyn Read>, Box<dyn Write>> {
		Interpreter::new(self.program, Box::new(self.input), Box::new(self.output))
	}

	#[inline]
	pub fn with_input<RR: Read>(self, input: RR) -> Interpreter<T, RR, W> {
		Interpreter::new(self.program, input, self.output)
	}

	#[inline]
	pub fn with_output<WW: Write>(self, output: WW) -> Interpreter<T, R, WW> {
		Interpreter::new(self.program, self.input, output)
	}

	#[inline]
	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Interpreter<T, RR, WW> {
		Interpreter::new(self.program, input, output)
	}

	#[inline]
	pub fn run(&mut self) -> Result<(), RuntimeError> {
		let program = mem::take(self.program_mut());

		program
			.into_iter()
			.try_for_each(|instr| self.execute_instruction(instr))
	}

	#[inline]
	fn read_char(&mut self) -> Result<(), RuntimeError> {
		loop {
			let mut buf = [0];
			let err = self.input.read_exact(&mut buf);
			match err.as_ref().map_err(IoError::kind) {
				Err(IoErrorKind::UnexpectedEof) => {
					mem::take(&mut buf);
				}
				_ => err?,
			}

			if cfg!(target_os = "windows") && matches!(buf[0], b'\r') {
				continue;
			}

			self.cell_mut().set_value(buf[0]);
			break;
		}

		Ok(())
	}

	#[inline]
	fn write_char(&mut self, count: usize, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		let ch = self.tape()[idx].value();

		self.write_to_output(ch, count)
	}

	#[inline]
	const fn boundary(&self) -> Result<(), RuntimeError> {
		Ok(())
	}

	#[inline]
	fn inc_val(&mut self, value: i8, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		WrappingAddAssign::wrapping_add_assign(&mut self.tape_mut()[idx], value);

		Ok(())
	}

	#[inline]
	fn set_val(&mut self, value: Option<NonZeroU8>, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		self.tape_mut()[idx].set_value(value.get_or_zero());

		Ok(())
	}

	#[inline]
	fn move_ptr(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		*self.ptr_mut() += offset.value();

		Ok(())
	}

	#[inline]
	fn find_zero(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		while !self.current_cell().is_zero() {
			self.move_ptr(offset)?;
		}

		Ok(())
	}

	#[inline]
	fn dyn_loop(&mut self, instructions: &[Instruction]) -> Result<(), RuntimeError> {
		let mut iterations = 0usize;

		while !self.current_cell().is_zero() {
			iterations += 1;

			if matches!(iterations, ITERATION_LIMIT) {
				return Err(RuntimeError::TooManyIterations(self.ptr().value()));
			}

			instructions
				.iter()
				.try_for_each(|instr| self.execute_instruction(instr))?;
		}

		Ok(())
	}

	#[inline]
	fn move_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let src_value = mem::take(self.cell_mut().as_mut_u8());

		let dst_offset = self.calculate_index(offset);

		WrappingAddAssign::wrapping_add_assign(&mut self.tape_mut()[dst_offset], src_value);

		Ok(())
	}

	#[inline]
	fn fetch_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(offset);

		let value = mem::take(self.tape_mut()[src_offset].as_mut_u8());

		WrappingAddAssign::wrapping_add_assign(self.cell_mut(), value);

		Ok(())
	}

	#[inline]
	fn scale_and_move_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.ptr().value();
		let dst_offset = self.calculate_index(offset);

		let tape = self.tape_mut();

		let src_val = mem::take(tape[src_offset].as_mut_u8());

		WrappingAddAssign::wrapping_add_assign(
			&mut tape[dst_offset],
			WrappingMul::wrapping_mul(src_val, factor),
		);

		Ok(())
	}

	#[inline]
	fn fetch_and_scale_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(offset);

		let value = mem::take(self.tape_mut()[src_offset].as_mut_u8());

		WrappingAddAssign::wrapping_add_assign(
			self.cell_mut(),
			WrappingMul::wrapping_mul(value, factor),
		);

		Ok(())
	}

	#[inline]
	fn scale_and_set_val(
		&mut self,
		factor: u8,
		offset: Offset,
		value: NonZeroU8,
	) -> Result<(), RuntimeError> {
		let src_offset = self.ptr().value();
		let dst_offset = self.calculate_index(offset);

		let tape = self.tape_mut();

		let src_val = mem::replace(tape[src_offset].as_mut_u8(), value.get());

		WrappingAddAssign::wrapping_add_assign(
			&mut tape[dst_offset],
			WrappingMul::wrapping_mul(src_val, factor),
		);

		Ok(())
	}

	#[inline]
	fn set_until_zero(
		&mut self,
		value: Option<NonZeroU8>,
		Offset(offset): Offset,
	) -> Result<(), RuntimeError> {
		while !self.current_cell().is_zero() {
			self.cell_mut().set_value(value.get_or_zero());
			*self.ptr_mut() += offset;
		}

		Ok(())
	}

	#[inline]
	fn scale_val(&mut self, factor: u8) -> Result<(), RuntimeError> {
		WrappingMulAssign::wrapping_mul_assign(self.cell_mut(), factor);

		Ok(())
	}

	#[inline]
	fn sub_cell(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		let current_value = mem::take(self.cell_mut().as_mut_u8());

		WrappingSubAssign::wrapping_sub_assign(&mut self.tape_mut()[idx], current_value);

		Ok(())
	}

	#[inline]
	fn take_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let current_value = mem::take(self.cell_mut().as_mut_u8());

		self.move_ptr(offset)?;

		WrappingAddAssign::wrapping_add_assign(self.cell_mut(), current_value);

		Ok(())
	}

	#[inline]
	fn scale_and_take_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let current_value = mem::take(self.cell_mut().as_mut_u8());

		self.move_ptr(offset)?;

		WrappingAddAssign::wrapping_add_assign(
			self.cell_mut(),
			WrappingMul::wrapping_mul(current_value, factor),
		);

		Ok(())
	}

	#[inline]
	fn dupe_val(&mut self, offsets: &[Offset]) -> Result<(), RuntimeError> {
		let value = mem::take(self.cell_mut().as_mut_u8());

		for offset in offsets {
			let idx = self.calculate_index(*offset);

			WrappingAddAssign::wrapping_add_assign(&mut self.tape_mut()[idx], value);
		}

		Ok(())
	}

	#[inline]
	fn find_and_set_zero(&mut self, offset: Offset, value: NonZeroU8) -> Result<(), RuntimeError> {
		self.find_zero(offset)?;

		self.set_val(Some(value), Offset(0))?;

		Ok(())
	}

	#[inline]
	fn find_cell_by_zero(&mut self, jump_by: Offset, offset: Offset) -> Result<(), RuntimeError> {
		self.find_zero(jump_by)?;

		self.move_ptr(offset)?;

		Ok(())
	}

	#[inline]
	fn replace_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(offset);

		let value = mem::take(self.tape_mut()[src_offset].as_mut_u8());

		_ = mem::replace(self.cell_mut().as_mut_u8(), value);

		Ok(())
	}

	#[inline]
	fn shift_vals(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		while !self.current_cell().is_zero() {
			self.move_val(offset)?;
			self.move_ptr(-offset)?;
		}

		Ok(())
	}

	#[inline]
	fn write_and_set(
		&mut self,
		count: usize,
		offset: Offset,
		value: Option<NonZeroU8>,
	) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		let ch = mem::replace(self.tape_mut()[idx].as_mut_u8(), value.get_or_zero());

		self.write_to_output(ch, count)
	}

	#[inline]
	fn write_byte(&mut self, b: u8) -> Result<(), RuntimeError> {
		self.write_to_output(b, 1)?;

		self.set_val(NonZeroU8::new(b), Offset(0))?;

		Ok(())
	}

	#[inline]
	fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), RuntimeError> {
		if let Some(profiler) = &mut self.profiler {
			profiler.handle(instr);
		}

		match instr {
			Instruction::Boundary => self.boundary()?,
			Instruction::IncVal { value, offset } => self.inc_val(*value, *offset)?,
			Instruction::SetVal { value, offset } => self.set_val(*value, *offset)?,
			Instruction::MovePtr(offset) => self.move_ptr(*offset)?,
			Instruction::Read => self.read_char()?,
			Instruction::FindZero(i) => self.find_zero(*i)?,
			Instruction::SubCell { offset } => self.sub_cell(*offset)?,
			Instruction::Block(l) => self.execute_loop_instruction(l)?,
			Instruction::ScaleVal { factor } => self.scale_val(*factor)?,
			Instruction::Super(s) => self.execute_super_instruction(*s)?,
			Instruction::FetchVal(offset) => self.fetch_val(*offset)?,
			Instruction::MoveVal(offset) => self.move_val(*offset)?,
			Instruction::DuplicateVal { offsets } => self.dupe_val(offsets)?,
			Instruction::TakeVal(offset) => self.take_val(*offset)?,
			Instruction::ReplaceVal(offset) => self.replace_val(*offset)?,
			Instruction::Write(w) => self.execute_write_instruction(w)?,
			i => return Err(RuntimeError::Unimplemented(i.clone())),
		}

		Ok(())
	}

	#[inline]
	fn execute_loop_instruction(&mut self, instr: &BlockInstruction) -> Result<(), RuntimeError> {
		match instr {
			BlockInstruction::DynamicLoop(instrs) => self.dyn_loop(instrs)?,
			BlockInstruction::IfNz(instrs) => {
				if self.current_cell().is_zero() {
					return Ok(());
				}

				instrs
					.iter()
					.try_for_each(|i| self.execute_instruction(i))?;

				mem::take(self.cell_mut().as_mut_u8());
			}
			i => return Err(RuntimeError::Unimplemented(i.clone().into())),
		}

		Ok(())
	}

	#[inline]
	fn execute_super_instruction(&mut self, instr: SuperInstruction) -> Result<(), RuntimeError> {
		match instr {
			SuperInstruction::ScaleAnd {
				action: ScaleAnd::Move,
				offset,
				factor,
			} => self.scale_and_move_val(factor, offset)?,
			SuperInstruction::ScaleAnd {
				action: ScaleAnd::Fetch,
				offset,
				factor,
			} => self.fetch_and_scale_val(factor, offset)?,
			SuperInstruction::ScaleAnd {
				action: ScaleAnd::Take,
				offset,
				factor,
			} => self.scale_and_take_val(factor, offset)?,
			SuperInstruction::ScaleAnd {
				action: ScaleAnd::Set(value),
				offset,
				factor,
			} => self.scale_and_set_val(factor, offset, value)?,
			SuperInstruction::FindAndSetZero { offset, value } => {
				self.find_and_set_zero(offset, value)?;
			}
			SuperInstruction::SetUntilZero { value, offset } => {
				self.set_until_zero(value, offset)?;
			}
			SuperInstruction::FindCellByZero { jump_by, offset } => {
				self.find_cell_by_zero(jump_by, offset)?;
			}
			SuperInstruction::ShiftVals(offset) => self.shift_vals(offset)?,
			i => return Err(RuntimeError::Unimplemented((i).into())),
		}

		Ok(())
	}

	fn execute_write_instruction(&mut self, instr: &WriteInstruction) -> Result<(), RuntimeError> {
		match instr {
			WriteInstruction::Cell { count, offset } => self.write_char(*count, *offset)?,
			WriteInstruction::CellAndSet {
				count,
				offset,
				value,
			} => self.write_and_set(*count, *offset, *value)?,
			WriteInstruction::Byte(ch) => self.write_byte(*ch)?,
			WriteInstruction::Bytes(s) => self.write_many_to_output(s)?,
			i => return Err(RuntimeError::Unimplemented(i.clone().into())),
		}

		Ok(())
	}

	#[inline]
	#[allow(clippy::missing_const_for_fn)]
	fn calculate_index(&self, offset: Offset) -> usize {
		match offset.0 {
			0 => self.ptr().value(),
			x => (self.ptr() + x).value(),
		}
	}

	#[inline]
	fn write_to_output(&mut self, ch: u8, count: usize) -> Result<(), RuntimeError> {
		let bytes = vec![ch; count];
		self.write_many_to_output(&bytes)
	}

	fn write_many_to_output(&mut self, bytes: &[u8]) -> Result<(), RuntimeError> {
		if bytes
			.iter()
			.all(|ch| !cfg!(target_os = "windows") || *ch < 128)
		{
			self.output.write_all(bytes)?;
			self.output.flush()?;
		}

		Ok(())
	}
}

impl<T: Tape> Interpreter<T, Stdin, Stdout> {
	#[inline]
	#[must_use]
	pub fn stdio(program: Program) -> Self {
		Self::new(program, stdin(), stdout())
	}
}

#[derive(Debug)]
pub enum RuntimeError {
	Io(IoError),
	Unimplemented(Instruction),
	TooManyIterations(usize),
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
			Self::Unimplemented(instr) => {
				f.write_str("instruction ")?;
				Debug::fmt(&instr, f)?;
				f.write_str(" is unimplmented")
			}
			Self::TooManyIterations(i) => {
				f.write_str("loop exceeded ")?;
				Display::fmt(&ITERATION_LIMIT, f)?;
				f.write_str(" iterations at cell ")?;
				Display::fmt(&i, f)
			}
		}
	}
}

impl StdError for RuntimeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::Unimplemented(_) | Self::TooManyIterations(_) => None,
		}
	}
}

impl From<IoError> for RuntimeError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}
