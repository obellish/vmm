#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod profiler;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind as IoErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
	mem,
	num::NonZeroU8,
};

use vmm_ir::{Instruction, LoopInstruction, Offset, ScaleAnd, SimdInstruction, SuperInstruction};
use vmm_program::Program;
use vmm_tape::{Tape, TapePointer};
use vmm_utils::GetOrZero as _;
use vmm_wrap::Wrapping;

pub use self::profiler::*;

pub const ITERATION_LIMIT: usize = 100_000;

#[derive(Debug, Clone)]
pub struct Interpreter<R = Stdin, W = Stdout> {
	program: Program,
	input: R,
	output: W,
	profiler: Option<Profiler>,
	tape: Tape,
}

impl<R, W> Interpreter<R, W> {
	pub fn new(program: Program, input: R, output: W) -> Self {
		Self {
			program,
			input,
			output,
			profiler: None,
			tape: Tape::new(),
		}
	}

	pub fn with_profiler(program: Program, input: R, output: W) -> Self {
		Self::new(program, input, output).and_with_profiler()
	}

	#[must_use]
	pub const fn and_with_profiler(mut self) -> Self {
		self.profiler = Some(Profiler::new());
		self
	}

	pub fn profiler(&self) -> Profiler {
		self.profiler.unwrap_or_default()
	}

	pub const fn program(&self) -> &Program {
		&self.program
	}

	pub const fn program_mut(&mut self) -> &mut Program {
		&mut self.program
	}

	pub const fn tape(&self) -> &Tape {
		&self.tape
	}

	pub const fn tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	pub fn cell(&self) -> &Wrapping<u8> {
		self.tape().cell()
	}

	pub fn cell_mut(&mut self) -> &mut Wrapping<u8> {
		self.tape_mut().cell_mut()
	}

	pub const fn ptr(&self) -> &TapePointer {
		self.tape().ptr()
	}

	pub const fn ptr_mut(&mut self) -> &mut TapePointer {
		self.tape_mut().ptr_mut()
	}
}

#[allow(clippy::unused_self)]
impl<R, W> Interpreter<R, W>
where
	R: Read + 'static,
	W: Write + 'static,
{
	pub fn into_dyn(self) -> Interpreter<Box<dyn Read>, Box<dyn Write>> {
		Interpreter::new(self.program, Box::new(self.input), Box::new(self.output))
	}

	pub fn with_input<RR: Read>(self, input: RR) -> Interpreter<RR, W> {
		Interpreter::new(self.program, input, self.output)
	}

	pub fn with_output<WW: Write>(self, output: WW) -> Interpreter<R, WW> {
		Interpreter::new(self.program, self.input, output)
	}

	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Interpreter<RR, WW> {
		Interpreter::new(self.program, input, output)
	}

	pub fn run(&mut self) -> Result<(), RuntimeError> {
		for instr in &std::mem::take(self.program_mut()) {
			self.execute_instruction(instr)?;
		}

		Ok(())
	}

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

			self.cell_mut().0 = buf[0];
			break;
		}

		Ok(())
	}

	fn write_char(&mut self, count: usize, offset: Option<Offset>) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		let ch = self.tape()[idx].0;

		let o = vec![ch; count];

		if !cfg!(target_os = "windows") || ch < 128 {
			self.output.write_all(&o)?;
			self.output.flush()?;
		}

		Ok(())
	}

	#[inline]
	const fn start(&self) -> Result<(), RuntimeError> {
		Ok(())
	}

	#[inline]
	fn inc_val(&mut self, value: i8, offset: Option<Offset>) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		self.tape_mut()[idx] += value;

		Ok(())
	}

	#[inline]
	fn set_val(
		&mut self,
		value: Option<NonZeroU8>,
		offset: Option<Offset>,
	) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		self.tape_mut()[idx].0 = value.get_or_zero();

		Ok(())
	}

	#[inline]
	fn move_ptr(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		match offset {
			Offset::Relative(i) => *self.ptr_mut() += i,
			Offset::Absolute(i) => self.ptr_mut().set(i),
		}

		Ok(())
	}

	#[inline]
	fn find_zero(&mut self, offset: isize) -> Result<(), RuntimeError> {
		while !matches!(self.cell().0, 0) {
			*self.ptr_mut() += offset;
		}

		Ok(())
	}

	#[inline]
	fn dyn_loop(&mut self, instructions: &[Instruction]) -> Result<(), RuntimeError> {
		let mut iterations = 0usize;

		while !matches!(self.cell().0, 0) {
			iterations += 1;

			if matches!(iterations, ITERATION_LIMIT) {
				return Err(RuntimeError::TooManyIterations(self.ptr().value()));
			}

			for instr in instructions {
				self.execute_instruction(instr)?;
			}
		}

		Ok(())
	}

	#[inline]
	fn scale_and_move_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.ptr().value();
		let dst_offset = self.calculate_index(Some(offset));

		let tape = self.tape_mut();

		let src_val = mem::take(&mut tape[src_offset]);

		tape[dst_offset] += (src_val * factor).0;

		Ok(())
	}

	#[inline]
	fn fetch_and_scale_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(Some(offset));

		let value = mem::take(&mut self.tape_mut()[src_offset]);

		*self.cell_mut() += (value * factor).0;

		Ok(())
	}

	#[inline]
	fn scale_val(&mut self, factor: u8) -> Result<(), RuntimeError> {
		*self.cell_mut() *= factor;

		Ok(())
	}

	#[inline]
	fn scale_and_take_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let current_value = mem::take(self.cell_mut());

		self.move_ptr(offset)?;

		*self.cell_mut() += Wrapping::mul(current_value.0, factor);

		Ok(())
	}

	#[inline]
	fn inc_vals(&mut self, value: i8, offsets: &[Option<Offset>]) -> Result<(), RuntimeError> {
		for offset in offsets {
			let idx = self.calculate_index(*offset);

			self.tape_mut()[idx] += value;
		}

		Ok(())
	}

	#[inline]
	fn set_vals(&mut self, v: Option<NonZeroU8>, offsets: &[Option<Offset>]) -> Result<(), RuntimeError> {
		for offset in offsets {
			let idx = self.calculate_index(*offset);

			self.tape_mut()[idx].0 = v.get_or_zero();
		}

		Ok(())
	}

	fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), RuntimeError> {
		if let Some(profiler) = &mut self.profiler {
			profiler.handle(instr);
		}

		match instr {
			Instruction::Start => self.start()?,
			Instruction::IncVal { value, offset } => self.inc_val(*value, *offset)?,
			Instruction::SetVal { value, offset } => self.set_val(*value, *offset)?,
			Instruction::MovePtr(offset) => self.move_ptr(*offset)?,
			Instruction::Write { offset, count } => self.write_char(*count, *offset)?,
			Instruction::Read => self.read_char()?,
			Instruction::FindZero(i) => self.find_zero(*i)?,
			Instruction::Loop(l) => self.execute_loop_instruction(l)?,
			Instruction::ScaleVal { factor } => self.scale_val(*factor)?,
			Instruction::Super(s) => self.execute_super_instruction(*s)?,
			Instruction::Simd(s) => self.execute_simd_instruction(s)?,
			i => return Err(RuntimeError::Unimplemented(i.clone())),
		}

		Ok(())
	}

	fn execute_loop_instruction(&mut self, instr: &LoopInstruction) -> Result<(), RuntimeError> {
		match instr {
			LoopInstruction::Dynamic(instrs) => self.dyn_loop(instrs)?,
			LoopInstruction::IfNz(instrs) => {
				if matches!(self.cell().0, 0) {
					return Ok(());
				}

				for i in instrs {
					self.execute_instruction(i)?;
				}
				mem::take(self.cell_mut());
			}
			i => return Err(RuntimeError::Unimplemented(i.clone().into())),
		}

		Ok(())
	}

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
			i => return Err(RuntimeError::Unimplemented(i.into())),
		}

		Ok(())
	}

	fn execute_simd_instruction(&mut self, instr: &SimdInstruction) -> Result<(), RuntimeError> {
		match instr {
			SimdInstruction::IncVals { value, offsets } => self.inc_vals(*value, offsets)?,
			SimdInstruction::SetVals { value, offsets } => self.set_vals(*value, offsets)?,
			i => return Err(RuntimeError::Unimplemented(i.clone().into())),
		}

		Ok(())
	}

	#[expect(clippy::missing_const_for_fn)]
	fn calculate_index(&self, offset: Option<Offset>) -> usize {
		match offset {
			None => self.ptr().value(),
			Some(Offset::Relative(offset)) => (*self.ptr() + offset).value(),
			Some(Offset::Absolute(i)) => {
				let mut ptr = *self.ptr();

				ptr.set(i);

				ptr.value()
			}
		}
	}
}

impl Interpreter<Stdin, Stdout> {
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
