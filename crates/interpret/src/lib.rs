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

use vmm_ir::{BlockInstruction, Instruction, Offset, ScaleAnd, SuperInstruction};
use vmm_num::Wrapping;
use vmm_program::Program;
use vmm_tape::{Tape, TapePointer};
use vmm_utils::GetOrZero as _;

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
	#[inline]
	pub fn new(program: Program, input: R, output: W) -> Self {
		Self {
			program,
			input,
			output,
			profiler: None,
			tape: Tape::new(),
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
	pub const fn tape(&self) -> &Tape {
		&self.tape
	}

	#[inline]
	pub const fn tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	#[inline]
	pub fn cell(&self) -> &Wrapping<u8> {
		self.tape().cell()
	}

	#[inline]
	pub fn cell_mut(&mut self) -> &mut Wrapping<u8> {
		self.tape_mut().cell_mut()
	}

	#[inline]
	pub const fn ptr(&self) -> &TapePointer {
		self.tape().ptr()
	}

	#[inline]
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
	#[inline]
	pub fn into_dyn(self) -> Interpreter<Box<dyn Read>, Box<dyn Write>> {
		Interpreter::new(self.program, Box::new(self.input), Box::new(self.output))
	}

	#[inline]
	pub fn with_input<RR: Read>(self, input: RR) -> Interpreter<RR, W> {
		Interpreter::new(self.program, input, self.output)
	}

	#[inline]
	pub fn with_output<WW: Write>(self, output: WW) -> Interpreter<R, WW> {
		Interpreter::new(self.program, self.input, output)
	}

	#[inline]
	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Interpreter<RR, WW> {
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

			self.cell_mut().0 = buf[0];
			break;
		}

		Ok(())
	}

	#[inline]
	fn write_char(&mut self, count: usize, offset: Offset) -> Result<(), RuntimeError> {
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
	fn inc_val(&mut self, value: i8, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		self.tape_mut()[idx] += value;

		Ok(())
	}

	#[inline]
	fn set_val(&mut self, value: Option<NonZeroU8>, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		self.tape_mut()[idx].0 = value.get_or_zero();

		Ok(())
	}

	#[inline]
	fn move_ptr(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		*self.ptr_mut() += offset.value();

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

			instructions
				.iter()
				.try_for_each(|instr| self.execute_instruction(instr))?;
		}

		Ok(())
	}

	#[inline]
	fn move_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let src_value = mem::take(self.cell_mut()).0;

		let dst_offset = self.calculate_index(offset);

		self.tape_mut()[dst_offset] += src_value;

		Ok(())
	}

	#[inline]
	fn fetch_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(offset);

		let value = mem::take(&mut self.tape_mut()[src_offset]).0;

		*self.cell_mut() += value;

		Ok(())
	}

	#[inline]
	fn scale_and_move_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.ptr().value();
		let dst_offset = self.calculate_index(offset);

		let tape = self.tape_mut();

		let src_val = mem::take(&mut tape[src_offset]);

		tape[dst_offset] += Wrapping::mul(src_val, factor).0;

		Ok(())
	}

	#[inline]
	fn fetch_and_scale_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(offset);

		let value = mem::take(&mut self.tape_mut()[src_offset]);

		*self.cell_mut() += Wrapping::mul(value, factor).0;

		Ok(())
	}

	#[inline]
	fn set_until_zero(
		&mut self,
		value: Option<NonZeroU8>,
		offset: isize,
	) -> Result<(), RuntimeError> {
		while !matches!(self.cell().0, 0) {
			_ = mem::replace(&mut self.cell_mut().0, value.get_or_zero());
			*self.ptr_mut() += offset;
		}

		Ok(())
	}

	#[inline]
	fn scale_val(&mut self, factor: u8) -> Result<(), RuntimeError> {
		*self.cell_mut() *= factor;

		Ok(())
	}

	#[inline]
	fn sub_cell(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let idx = self.calculate_index(offset);

		let current_value = mem::take(self.cell_mut());

		self.tape_mut()[idx] -= current_value.0;

		Ok(())
	}

	#[inline]
	fn take_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let current_value = mem::take(self.cell_mut()).0;

		self.move_ptr(offset)?;

		*self.cell_mut() += current_value;

		Ok(())
	}

	#[inline]
	fn scale_and_take_val(&mut self, factor: u8, offset: Offset) -> Result<(), RuntimeError> {
		let current_value = mem::take(self.cell_mut());

		self.move_ptr(offset)?;

		*self.cell_mut() += Wrapping::mul(current_value, factor).0;

		Ok(())
	}

	#[inline]
	fn dupe_val(&mut self, offsets: &[Offset]) -> Result<(), RuntimeError> {
		let value = mem::take(self.cell_mut()).0;

		for offset in offsets {
			let idx = self.calculate_index(*offset);

			self.tape_mut()[idx] += value;
		}

		Ok(())
	}

	#[inline]
	fn find_and_set_zero(&mut self, offset: isize, value: NonZeroU8) -> Result<(), RuntimeError> {
		while !matches!(self.cell().0, 0) {
			*self.ptr_mut() += offset;
		}

		self.cell_mut().0 = value.get();

		Ok(())
	}

	#[inline]
	fn replace_val(&mut self, offset: Offset) -> Result<(), RuntimeError> {
		let src_offset = self.calculate_index(offset);

		let value = mem::take(&mut self.tape_mut()[src_offset]);

		_ = mem::replace(self.cell_mut(), value);

		Ok(())
	}

	#[inline]
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
			Instruction::SubCell { offset } => self.sub_cell(*offset)?,
			Instruction::Block(l) => self.execute_loop_instruction(l)?,
			Instruction::ScaleVal { factor } => self.scale_val(*factor)?,
			Instruction::Super(s) => self.execute_super_instruction(*s)?,
			Instruction::FetchVal(offset) => self.fetch_val(*offset)?,
			Instruction::MoveVal(offset) => self.move_val(*offset)?,
			Instruction::DuplicateVal { offsets } => self.dupe_val(offsets)?,
			Instruction::TakeVal(offset) => self.take_val(*offset)?,
			Instruction::ReplaceVal(offset) => self.replace_val(*offset)?,
			i => return Err(RuntimeError::Unimplemented(i.clone())),
		}

		Ok(())
	}

	#[inline]
	fn execute_loop_instruction(&mut self, instr: &BlockInstruction) -> Result<(), RuntimeError> {
		match instr {
			BlockInstruction::DynamicLoop(instrs) => self.dyn_loop(instrs)?,
			BlockInstruction::IfNz(instrs) => {
				if matches!(self.cell().0, 0) {
					return Ok(());
				}

				instrs
					.iter()
					.try_for_each(|i| self.execute_instruction(i))?;

				mem::take(self.cell_mut());
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
			SuperInstruction::FindAndSetZero { offset, value } => {
				self.find_and_set_zero(offset, value)?;
			}
			SuperInstruction::SetUntilZero { value, offset } => {
				self.set_until_zero(value, offset)?;
			}
			i => return Err(RuntimeError::Unimplemented((i).into())),
		}

		Ok(())
	}

	#[inline]
	#[allow(clippy::missing_const_for_fn)]
	fn calculate_index(&self, offset: Offset) -> usize {
		match offset.0 {
			0 => self.ptr().value(),
			x => (*self.ptr() + x).value(),
		}
	}
}

impl Interpreter<Stdin, Stdout> {
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
