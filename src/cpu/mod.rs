mod regs;

pub use self::regs::*;
use super::{
	board::HardwareBridge,
	mem::MappedMemory,
	mmu::{MemAction, Mmu},
};

#[derive(Debug)]
pub struct Cpu {
	pub regs: Registers,
	pub(crate) mem: MappedMemory,
	mmu: Mmu,
	hwb: HardwareBridge,
	cycles: u128,
	halted: bool,
	_cycle_changed_pc: bool,
}

impl Cpu {
	#[must_use]
	pub const fn new(hwb: HardwareBridge, mem: MappedMemory) -> Self {
		let mut cpu = Self {
			regs: Registers::new(),
			mem,
			mmu: Mmu::new(),
			hwb,
			cycles: 0,
			halted: true,
			_cycle_changed_pc: false,
		};

		cpu.regs.smt = 1;

		cpu
	}

	pub fn reset(&mut self) {
		self.regs.reset();
		self.regs.smt = 1;
		self.cycles = 0;
		self.halted = false;
		self._cycle_changed_pc = true;
	}

	pub fn next(&mut self) {
		if self.halted {
			return;
		}

		self.cycles = self.cycles.wrapping_add(1);

		let Some(instr) = self.mem_exec(self.regs.pc).map(u32::to_be_bytes) else {
			return;
		};

		let opcode = instr[0] >> 3;
		let opregs = [
			!matches!(instr[0] & 0b100, 0),
			!matches!(instr[0] & 0b10, 0),
			!matches!(instr[0] & 0b1, 0),
		];

		let params = [instr[1], instr[2], instr[3]];

		self._cycle_changed_pc = false;

		if self.run_instr(opcode, opregs, params).is_none() {
			return;
		}

		if !self._cycle_changed_pc {
			self.regs.pc = self.regs.pc.wrapping_add(4);
		}
	}

	#[must_use]
	pub const fn halted(&self) -> bool {
		self.halted
	}

	#[must_use]
	pub const fn cycles(&self) -> u128 {
		self.cycles
	}

	fn run_instr(&mut self, opcode: u8, opregs: [bool; 3], params: [u8; 3]) -> Option<()> {
		macro_rules! args {
			(REG) => {
				params[0]
			};
			(REG_OR_LIT_1) => {
				__reg_or_lit!(0, 1)
			};
			(REG_OR_LIT_2) => {
				__reg_or_lit!(0, 2)
			};
			(REG, REG) => {
				(params[0], params[1])
			};
			(REG, REG_OR_LIT_1) => {
				(params[0], __reg_or_lit!(1, 1))
			};
			(REG, REG_OR_LIT_2) => {
				(params[0], __reg_or_lit!(1, 2))
			};
			(REG_OR_LIT_1, REG) => {
				(__reg_or_lit!(0, 1), params[1])
			};
			(REG_OR_LIT_1, REG_OR_LIT_1) => {
				(__reg_or_lit!(0, 1), __reg_or_lit!(1, 1))
			};
			(REG_OR_LIT_1, REG_OR_LIT_2) => {
				(__reg_or_lit!(0, 1), __reg_or_lit!(1, 2))
			};
			(REG, REG, REG) => {
				(params[0], params[1], params[2])
			};
			(REG, REG, REG_OR_LIT_1) => {
				(params[0], params[1], __reg_or_lit!(2, 1))
			};
			(REG, REG_OR_LIT_1, REG) => {
				(params[0], __reg_or_lit!(1, 1), params[2])
			};
			(REG, REG_OR_LIT_1, REG_OR_LIT_1) => {
				(params[0], __reg_or_lit!(1, 1), __reg_or_lit!(2, 1))
			};
			(REG_OR_LIT_1, REG, REG) => {
				(__reg_or_lit!(0, 1), params[1], params[2])
			};
			(REG_OR_LIT_1, REG, REG_OR_LIT_1) => {
				(__reg_or_lit!(0, 1), params[1], __reg_or_lit!(2, 1))
			};
			(REG_OR_LIT_1, REG_OR_LIT_1, REG) => {
				(__reg_or_lit!(0, 1), __reg_or_lit!(1, 1), params[2])
			};
			(REG_OR_LIT_1, REG_OR_LIT_1, REG_OR_LIT_1) => {
				(
					__reg_or_lit!(0, 1),
					__reg_or_lit!(1, 1),
					__reg_or_lit!(2, 1),
				)
			};
		}

		macro_rules! __reg_or_lit {
            ($param:expr, 1) => {
                __reg_or_lit!(@with_val $param, params[$param].into())
            };
            ($param: expr, 2) => {
                __reg_or_lit!(@with_val $param, u16::from_be_bytes([params[$param], params[$param + 1]]).into())
            };
            (@with_val $param:expr, $literal:expr) => {
                if opregs[$param] {
                    self.read_reg(params[$param])?
                } else {
                    $literal
                }
            };
        }

		match opcode {
			0x00 => {
				self.exception(0x01, Some(opcode.into()));
				None
			}
			0x01 => {
				let (reg_dest, value) = args!(REG, REG_OR_LIT_2);
				self.write_reg(reg_dest, value).then_some(())
			}
			0x02 => {
				let (reg_a, reg_b) = args!(REG, REG);
				let pivot_a = self.read_reg(reg_a)?;

				self.read_reg(reg_b).and_then(|reg_b_value| {
					self.write_reg(reg_a, reg_b_value).then_some(())?;
					self.write_reg(reg_b, pivot_a).then_some(())
				})
			}
			0x03..=0x05 | 0x08..=0x0C => {
				let (reg, mut value) = args!(REG, REG_OR_LIT_2);

				if matches!(opcode, 0x0B | 0x0C) && !opregs[1] {
					value >>= 8;
				}

				let reg_value = self.read_reg(reg)?;

				let compute = self.compute(reg_value, value, match opcode {
					0x03 => Op::Add,
					0x04 => Op::Sub,
					0x05 => Op::Mul,
					0x08 => Op::And,
					0x09 => Op::Bor,
					0x0A => Op::Xor,
					0x0B => Op::Shl,
					0x0C => Op::Shr,
					_ => unreachable!(),
				})?;

				self.write_reg(reg, compute).then_some(())
			}
			0x06 => {
				let (reg, value, mode) = args!(REG, REG_OR_LIT_1, REG_OR_LIT_1);
				let reg_value = self.read_reg(reg)?;

				let compute = self.compute(reg_value, value, Op::Div {
					mode: (mode & 0xFF) as u8,
				})?;
				self.write_reg(reg, compute).then_some(())
			}
			0x07 => {
				let (reg, value, mode) = args!(REG, REG_OR_LIT_1, REG_OR_LIT_1);
				let reg_value = self.read_reg(reg)?;

				let compute = self.compute(reg_value, value, Op::Mod {
					mode: (mode & 0xFF) as u8,
				})?;
				self.write_reg(reg, compute).then_some(())
			}
			0x0D => {
				let (reg, value) = args!(REG, REG_OR_LIT_2);
				let reg_value = self.read_reg(reg)?;

				self.compute(reg_value, value, Op::Sub)?;

				Some(())
			}
			0x0E => {
				let bytes = args!(REG_OR_LIT_2) as i16;

				self.regs.pc = self.regs.pc.wrapping_add(bytes as u32);
				self._cycle_changed_pc = true;
				Some(())
			}
			0x0F => {
				if self.sv_mode() {
					self.regs.pc = args!(REG_OR_LIT_2);
					self.regs.smt = 0;
					Some(())
				} else {
					self.exception(0x09, Some(opcode.into()));
					None
				}
			}
			0x10 => {
				let itr_code = args!(REG_OR_LIT_1);

				self.exception(0xF0, Some(itr_code as u16));
				Some(())
			}
			0x11 | 0x12 => {
				let flag = args!(REG_OR_LIT_1);

				if flag > 7 {
					self.exception(0x0C, Some(u16::from(flag as u8)));
					return None;
				}

				let is_flag_set = !matches!(self.regs.af & (1 << (7 - flag)), 0);

				if is_flag_set != matches!(opcode, 0x11) {
					self.regs.pc = self.regs.pc.wrapping_add(4);
				}

				Some(())
			}
			0x13 => {
				let (flag_a, flag_b, cond) = args!(REG_OR_LIT_1, REG_OR_LIT_1, REG_OR_LIT_1);
				let (flag_a, flag_b) = (
					!matches!(self.regs.af & (1 << (7 - flag_a)), 0),
					!matches!(self.regs.af & (1 << (7 - flag_b)), 0),
				);

				let result = match cond {
					0x01 => flag_a || flag_b,
					0x02 => flag_a && flag_b,
					0x03 => flag_a ^ flag_b,
					0x04 => !flag_a && !flag_b,
					0x05 => !(flag_a && flag_b),
					0x06 => flag_a && !flag_b,
					0x07 => !flag_a && flag_b,
					_ => {
						self.exception(0x0D, Some(u16::from(cond as u8)));
						return None;
					}
				};

				if !result {
					self.regs.pc = self.regs.pc.wrapping_add(4);
				}

				Some(())
			}
			0x14 => {
				let (reg_dest, v_addr, add) = args!(REG, REG_OR_LIT_1, REG_OR_LIT_1);

				let word = self.mem_read(v_addr.wrapping_add(add))?;

				self.write_reg(reg_dest, word).then_some(())
			}
			0x15 => {
				let (v_addr, add, mul) = args!(REG_OR_LIT_1, REG_OR_LIT_1, REG_OR_LIT_1);

				self.regs.avr = self.mem_read(v_addr.wrapping_add(add.wrapping_mul(mul)))?;

				Some(())
			}
			0x16 => {
				let (v_addr, add, val) = args!(REG_OR_LIT_1, REG_OR_LIT_1, REG_OR_LIT_1);

				self.mem_write(v_addr.wrapping_add(add), val)
			}
			0x17 => {
				let (v_addr, add, mul) = args!(REG_OR_LIT_1, REG_OR_LIT_1, REG_OR_LIT_1);

				self.mem_write(v_addr.wrapping_add(add.wrapping_mul(mul)), self.regs.avr)
			}
			0x18 => {
				let (v_addr, add, reg_swap) = args!(REG_OR_LIT_1, REG_OR_LIT_1, REG);

				let old_word = self.mem_read(v_addr + add)?;
				let to_write = self.read_reg(reg_swap)?;
				self.mem_write(v_addr + add, to_write)?;
				self.write_reg(reg_swap, old_word).then_some(())
			}
			0x19 => {
				let word = args!(REG_OR_LIT_2);

				let stack_v_addr = if self.sv_mode() {
					self.regs.ssp
				} else {
					self.regs.usp
				}
				.wrapping_sub(4);

				self.mem_write(stack_v_addr, word)?;

				if self.sv_mode() {
					self.regs.ssp = stack_v_addr;
				} else {
					self.regs.usp = stack_v_addr;
				}

				Some(())
			}
			0x1A => {
				let reg_dest = args!(REG);

				let word = if self.sv_mode() {
					let word = self.mem_read(self.regs.ssp)?;
					self.regs.ssp = self.regs.ssp.wrapping_add(4);
					word
				} else {
					let word = self.mem_read(self.regs.usp)?;
					self.regs.usp = self.regs.usp.wrapping_add(4);
					word
				};

				self.write_reg(reg_dest, word).then_some(())
			}
			0x1B => {
				let jmp_v_addr = args!(REG_OR_LIT_2);

				let stack_v_addr = if self.sv_mode() {
					self.regs.ssp
				} else {
					self.regs.usp
				}
				.wrapping_sub(4);

				self.mem_write(stack_v_addr, self.regs.pc + 4)?;

				if self.sv_mode() {
					self.regs.ssp = stack_v_addr;
				} else {
					self.regs.usp = stack_v_addr;
				}

				self.regs.pc = jmp_v_addr;
				self._cycle_changed_pc = true;

				Some(())
			}
			0x1C => {
				let (reg_dest, aux_id, hw_info) = args!(REG, REG_OR_LIT_1, REG_OR_LIT_1);

				if matches!((aux_id, hw_info), (0, 0)) {
					return self
						.write_reg(reg_dest, self.hwb.count() as u32)
						.then_some(());
				}

				let Ok(aux_id) = usize::try_from(aux_id) else {
					self.exception(0x10, Some(aux_id as u16));
					return None;
				};

				let hw_data = self.get_hw_info(hw_info, aux_id)?;

				self.write_reg(reg_dest, hw_data).then_some(())
			}
			0x1D => {
				let reg_dest = args!(REG);
				self.write_reg(reg_dest, self.cycles as u32).then_some(())
			}
			0x1E => {
				self.halted = true;
				Some(())
			}
			0x1F => {
				let mode = args!(REG_OR_LIT_1);

				let (cpu_mode, aux_mode) = ((mode & 0xF0) as u8, (mode & 0x0F) as u8);

				match aux_mode {
					0x0 => {
						for id in 0..self.hwb.count() {
							self.hwb.reset(id)?;
						}
					}
					0x1 => {
						let Ok(id) = usize::try_from(self.regs.avr) else {
							self.exception(0x10, Some(self.regs.avr as u16));
							return None;
						};
						if self.hwb.reset(id).is_none() {
							self.exception(0x10, Some(self.regs.avr as u16));
							return None;
						}
					}
					0x2..=0x4 => {
						let ignore_id = usize::try_from(self.regs.avr).ok();

						let test = move |id| {
							ignore_id.is_none_or(|ignore_id| match aux_mode {
								0x2 => id != ignore_id,
								0x3 => id < ignore_id,
								0x4 => id > ignore_id,
								_ => unreachable!(),
							})
						};

						for id in 0..self.hwb.count() {
							if test(id) {
								self.hwb.reset(id)?;
							}
						}
					}
					_ => {}
				}

				if matches!(cpu_mode, 0) {
					self.reset();
				}

				Some(())
			}
			_ => unreachable!(
				"internal error: processor encountered an instruction with an opcode greater than 0x1F (> 5 bits)"
			),
		}
	}

	fn read_reg(&mut self, code: u8) -> Option<u32> {
		if code >= 0x18 && !self.sv_mode() {
			self.exception(0x03, Some(code.into()));
			return None;
		}

		match code {
			0x00..=0x07 => Some(self.regs.a[usize::from(code)]),
			0x08..=0x09 => Some(self.regs.c[usize::from(code) - 0x08]),
			0x0A..=0x0C => Some(self.regs.ac[usize::from(code) - 0x0A]),
			0x0D..=0x14 => Some(self.regs.rr[usize::from(code) - 0x0D]),
			0x15 => Some(self.regs.avr),
			0x16 => Some(self.regs.pc),
			0x17 => Some(self.regs.af),
			0x18 => Some(self.regs.ssp),
			0x19 => Some(self.regs.usp),
			0x1A => Some(self.regs.et),
			0x1B => Some(self.regs.era),
			0x1C => Some(self.regs.ev),
			0x1D => Some(self.regs.mtt),
			0x1E => Some(self.regs.pda),
			0x1F => Some(self.regs.smt),
			_ => {
				self.exception(0x02, Some(code.into()));
				None
			}
		}
	}

	fn write_reg(&mut self, code: u8, word: u32) -> bool {
		let ucode = usize::from(code);

		if code >= 0x17 && !self.sv_mode() {
			self.exception(0x04, Some(code.into()));
			return false;
		}

		if matches!(code, 0x17 | 0x1A | 0x1B) {
			self.exception(0x04, Some(code.into()));
			return false;
		}

		if matches!(code, 0x16) {
			self._cycle_changed_pc = true;
		}

		match code {
			0x00..=0x07 => self.regs.a[ucode] = word,
			0x08..=0x09 => self.regs.c[ucode - 0x08] = word,
			0x0A..=0x0C => self.regs.ac[ucode - 0x0A] = word,
			0x0D..=0x14 => self.regs.rr[ucode - 0x0D] = word,
			0x15 => self.regs.avr = word,
			0x16 => self.regs.pc = word,
			0x17 => self.regs.af = word,
			0x18 => self.regs.ssp = word,
			0x19 => self.regs.usp = word,
			0x1A => self.regs.et = word,
			0x1B => self.regs.era = word,
			0x1C => self.regs.ev = word,
			0x1D => self.regs.mtt = word,
			0x1E => self.regs.pda = word,
			0x1F => self.regs.smt = word,
			_ => {
				self.exception(0x02, Some(code.into()));
				return false;
			}
		}

		true
	}

	fn compute(&mut self, op1: u32, op2: u32, op: Op) -> Option<u32> {
		let iop1 = op1 as i32;
		let iop2 = op2 as i32;

		let (result, has_carry, has_overflow) = match op {
			Op::Add => {
				let (result, has_carry) = op1.overflowing_add(op2);
				(result, has_carry, iop1.overflowing_add(iop2).1)
			}
			Op::Sub => {
				let (result, has_carry) = op1.overflowing_sub(op2);
				(result, has_carry, iop1.overflowing_sub(iop2).1)
			}
			Op::Mul => {
				let (result, has_carry) = iop1.overflowing_mul(iop2);
				(result as u32, has_carry, has_carry)
			}
			Op::Div { mode } | Op::Mod { mode } => {
				let signed = !matches!(mode & 0b0001_0000, 0);

				match (op == Op::Div { mode }, signed, iop1, iop2) {
					(_, _, _, 0) => match (mode & 0b0000_1100) >> 2 {
						0b00 => {
							self.exception(0x0A, None);
							return None;
						}
						0b01 => (0x8000_0000, true, true),
						0b10 => (0x0000_0000, true, true),
						0b11 => (0x7FFF_FFFF, true, true),
						_ => unreachable!(),
					},
					(_, true, i32::MIN, -1) => match (mode & 0b0000_0011) >> 2 {
						0b00 => {
							self.exception(0x0B, None);
							return None;
						}
						0b01 => (0x8000_0000, true, true),
						0b10 => (0x0000_0000, true, true),
						0b11 => (0x7FFF_FFFF, true, true),
						_ => unreachable!(),
					},
					(true, true, _, _) => ((iop1 / iop2) as u32, false, false),
					(false, true, _, _) => ((iop1 % iop2) as u32, false, false),
					(true, false, _, _) => (op1 / op2, false, false),
					(false, false, _, _) => (op1 % op2, false, false),
				}
			}
			Op::And => (op1 & op2, false, false),
			Op::Bor => (op1 | op2, false, false),
			Op::Xor => (op1 ^ op2, false, false),
			Op::Shl => {
				let (result, has_carry) = op1.overflowing_shl(op2);
				(result, has_carry, has_carry)
			}
			Op::Shr => {
				let (result, has_carry) = op1.overflowing_shr(op2);
				(result, has_carry, has_carry)
			}
		};

		self.regs.af = 0;

		let flags = [
			matches!(result, 0),
			has_carry,
			has_overflow,
			matches!((result >> 31) & 0b1, 1),
			matches!(result & 0b1, 0),
			result <= 0xFFFF,
			matches!((result >> 16).trailing_zeros(), 0),
		];

		for (bit, flag) in flags.iter().copied().enumerate() {
			if flag {
				self.regs.af += 1 << (7 - bit);
			}
		}

		Some(result)
	}

	const fn sv_mode(&self) -> bool {
		!matches!(self.regs.smt, 0)
	}

	fn exception(&mut self, code: u8, associated: Option<u16>) {
		self.regs.et = (if self.sv_mode() { 1 << 24 } else { 0 })
			+ (u32::from(code) << 16)
			+ u32::from(associated.unwrap_or(0));

		self.regs.pc = self.regs.ev;

		self.regs.smt = 1;

		self._cycle_changed_pc = true;
	}

	fn ensure_aligned(&mut self, v_addr: u32) -> Option<u32> {
		if matches!(v_addr % 4, 0) {
			Some(v_addr)
		} else {
			self.exception(0x05, Some((v_addr % 4) as u16));
			None
		}
	}

	fn mem_do<T: std::fmt::Debug>(
		&mut self,
		action: MemAction,
		v_addr: u32,
		mut handler: impl FnMut(&mut MappedMemory, u32, &mut u16) -> T,
	) -> Option<T> {
		let v_addr = self.ensure_aligned(v_addr)?;

		match self
			.mmu
			.translate(&mut self.mem, &self.regs, v_addr, action)
		{
			Ok(p_addr) => {
				let mut ex = 0;
				let ret = handler(&mut self.mem, p_addr, &mut ex);

				if matches!(ex, 0) {
					Some(ret)
				} else {
					self.exception(0xA0, Some(ex));
					None
				}
			}
			Err(None) => {
				self.exception(0x06, Some(v_addr as u16));
				None
			}
			Err(Some(ex)) => {
				self.exception(0xA0, Some(ex));
				None
			}
		}
	}

	fn mem_read(&mut self, v_addr: u32) -> Option<u32> {
		self.mem_do(MemAction::Read, v_addr, |mem, p_addr, ex| {
			mem.read(p_addr, ex)
		})
	}

	fn mem_write(&mut self, v_addr: u32, word: u32) -> Option<()> {
		self.mem_do(MemAction::Write, v_addr, |mem, p_addr, ex| {
			mem.write(p_addr, word, ex);
		})
	}

	fn mem_exec(&mut self, v_addr: u32) -> Option<u32> {
		self.mem_do(MemAction::Exec, v_addr, |mem, p_addr, ex| {
			mem.read(p_addr, ex)
		})
	}

	fn get_hw_info(&mut self, hw_info: u32, aux_id: usize) -> Option<u32> {
		let Some(cache) = self.hwb.cache_of(aux_id).cloned() else {
			self.exception(0x10, Some(aux_id as u16));

			return None;
		};

		let mapping_opt = self.mem.get_mapping(aux_id).copied();

		let aux_name = cache.name.bytes();

		let data = match hw_info {
			0x01 => cache.metadata[0],
			0x02 => cache.metadata[1],
			0x10 => aux_name.count() as u32,
			0x11..=0x18 => {
				let mut name_bytes = aux_name.skip(((hw_info - 0x11) * 4) as usize);
				u32::from_be_bytes([
					name_bytes.next().unwrap_or(0),
					name_bytes.next().unwrap_or(0),
					name_bytes.next().unwrap_or(0),
					name_bytes.next().unwrap_or(0),
				])
			}
			0x20 => cache.metadata[2],
			0x21 => cache.metadata[3],
			0x22 => cache.metadata[4],
			0x23 => cache.metadata[5],
			0x24 => cache.metadata[6],
			0x25 => cache.metadata[7],
			0xA0 => u32::from(mapping_opt.is_some()),
			0xA1 => {
				if let Some(opt) = mapping_opt {
					opt.addr
				} else {
					self.exception(0x12, Some(aux_id as u16));
					return None;
				}
			}
			0xA2 => {
				if let Some(opt) = mapping_opt {
					opt.end_addr()
				} else {
					self.exception(0x12, Some(aux_id as u16));
					return None;
				}
			}
			_ => {
				self.exception(0x11, Some(hw_info as u16));
				return None;
			}
		};

		Some(data)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
	Add,
	Sub,
	Mul,
	Div { mode: u8 },
	Mod { mode: u8 },
	And,
	Bor,
	Xor,
	Shl,
	Shr,
}
