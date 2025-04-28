use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
	mem,
};

use super::{ExecutionUnit, Instruction};

pub struct Vm<R = Stdin, W = Stdout>
where
	R: Read + 'static,
	W: Write + 'static,
{
	unit: ExecutionUnit,
	input: R,
	output: W,
	counter: usize,
}

impl<R: Read, W: Write> Vm<R, W> {
	pub const fn new(unit: ExecutionUnit, input: R, output: W) -> Self {
		Self {
			unit,
			input,
			output,
			counter: 0,
		}
	}

	pub fn into_dyn(self) -> Vm<Box<dyn Read>, Box<dyn Write>> {
		Vm::new(self.unit, Box::new(self.input), Box::new(self.output))
	}

	pub fn run(mut self) -> Result<(), RuntimeError> {
		// for instr in mem::take(self.unit.program_mut()).iter().copied() {
		// 	match instr {
		// 		Instruction::Add(i) => *self.unit.tape_mut().current_cell_mut() = self.unit.tape().current_cell().wrapping_add(i as u8),
		// 		Instruction::Move(i) => {
		// 			if i > 0 {
		// 				self.unit.tape_mut().increment_pointer(i.unsigned_abs());
		// 			} else {
		// 				self.unit.tape_mut().decrement_pointer(i.unsigned_abs());
		// 			}
		// 		}
		// 		Instruction::Read => self.read_char()?,
		// 		Instruction::Write => self.write_char()?,
		// 		_ => {}
		// 	}
		// }
		'program: loop {
			match *self.current_instruction() {
				Instruction::Add(i) => {
					*self.unit.tape_mut().current_cell_mut() =
						self.unit.tape().current_cell().wrapping_add(i as u8);
				}
				Instruction::Move(i) => {
					if i > 0 {
						self.unit.tape_mut().increment_pointer(i.unsigned_abs());
					} else {
						self.unit.tape_mut().decrement_pointer(i.unsigned_abs());
					}
				}
				Instruction::Read => self.read_char()?,
				Instruction::Write => self.write_char()?,
				Instruction::JumpLeft => {
					if !matches!(self.unit.tape().current_cell(), 0) {
						let mut deep = 1;

						loop {
							if matches!(self.counter, 0) {
								break 'program;
							}

							self.counter -= 1;

							if matches!(self.current_instruction(), Instruction::JumpLeft) {
								deep += 1;
							}

							if matches!(self.current_instruction(), Instruction::JumpRight) {
								deep -= 1;
							}

							if matches!(deep, 0) {
								break;
							}
						}
					}
				}
				Instruction::JumpRight => {
					if matches!(self.unit.tape().current_cell(), 0) {
						let mut deep = 1;

						loop {
							if self.counter + 1 == self.unit.program().len() {
								break 'program;
							}

							self.counter += 1;
							if matches!(self.current_instruction(), Instruction::JumpRight) {
								deep += 1;
							}

							if matches!(self.current_instruction(), Instruction::JumpLeft) {
								deep -= 1;
							}

							if matches!(deep, 0) {
								break;
							}
						}
					}
				}
			}

			self.counter += 1;

			if self.unit.program().len() == self.counter {
				break 'program;
			}
		}

		Ok(())
	}

	pub fn with_input<RR: Read>(self, input: RR) -> Vm<RR, W> {
		Vm::new(self.unit, input, self.output)
	}

	pub fn with_output<WW: Write>(self, output: WW) -> Vm<R, WW> {
		Vm::new(self.unit, self.input, output)
	}

	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Vm<RR, WW> {
		Vm::new(self.unit, input, output)
	}

	fn read_char(&mut self) -> Result<(), RuntimeError> {
		let mut buf = [0];
		if let Err(e) = self.input.read_exact(&mut buf) {
			if !matches!(e.kind(), ErrorKind::UnexpectedEof) {
				return Err(e.into());
			}
		}

		*self.unit.tape_mut().current_cell_mut() = buf[0];

		Ok(())
	}

	fn write_char(&mut self) -> Result<(), RuntimeError> {
		let ch = self.unit.tape().current_cell();

		if ch.is_ascii() {
			self.output.write_all(&[ch])?;
		} else {
			write!(self.output, "\\0x{ch:x}")?;
		}

		self.output.flush()?;

		Ok(())
	}

	fn current_instruction(&self) -> &Instruction {
		unsafe { self.unit.program().get_unchecked(self.counter) }
	}
}

impl Vm<Stdin, Stdout> {
	#[must_use]
	pub fn stdio(unit: ExecutionUnit) -> Self {
		Self::new(unit, stdin(), stdout())
	}
}

#[derive(Debug)]
pub enum RuntimeError {
	Io(IoError),
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for RuntimeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
		}
	}
}

impl From<IoError> for RuntimeError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}
