mod ext;

use std::borrow::Cow;

use thiserror::Error;

pub use self::ext::ExtInstr;
use super::{ArFlag, DivMode, HwInfo, If2Cond, Reg, RegOrLit1, RegOrLit2, ToLasm, cst};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instr {
	Cpy(Reg, RegOrLit2),
	Ex(Reg, Reg),
	Add(Reg, RegOrLit2),
	Sub(Reg, RegOrLit2),
	Mul(Reg, RegOrLit2),
	Div(Reg, RegOrLit1, RegOrLit1),
	Mod(Reg, RegOrLit1, RegOrLit1),
	And(Reg, RegOrLit2),
	Bor(Reg, RegOrLit2),
	Xor(Reg, RegOrLit2),
	Shl(Reg, RegOrLit1),
	Shr(Reg, RegOrLit1),
	Cmp(Reg, RegOrLit2),
	Jpr(RegOrLit2),
	Lsm(RegOrLit2),
	Itr(RegOrLit1),
	If(RegOrLit1),
	IfN(RegOrLit1),
	If2(RegOrLit1, RegOrLit1, RegOrLit1),
	Lsa(Reg, RegOrLit1, RegOrLit1),
	Lea(RegOrLit1, RegOrLit1, RegOrLit1),
	Wsa(RegOrLit1, RegOrLit1, RegOrLit1),
	Wea(RegOrLit1, RegOrLit1, RegOrLit1),
	Srm(RegOrLit1, RegOrLit1, Reg),
	Push(RegOrLit2),
	Pop(Reg),
	Call(RegOrLit2),
	Hwd(Reg, RegOrLit1, RegOrLit1),
	Cycles(Reg),
	Halt,
	Reset(RegOrLit1),
}

impl Instr {
	pub fn decode(bytes: [u8; 4]) -> Result<Self, InstrDecodingError> {
		let opcode = bytes[0] >> 3;

		let (arg_reg, arg_reg_or_lit_1, arg_reg_or_lit_2) = {
			let mut _decode_reg = move |param: usize| {
				Reg::from_code(bytes[param]).ok_or_else(|| InstrDecodingError::UnknownRegister {
					param: param - 1,
					code: bytes[param],
				})
			};

			(
				_decode_reg,
				move |param: usize| -> Result<RegOrLit1, InstrDecodingError> {
					if matches!(bytes[0] & (1 << (3 - param)), 0) {
						Ok(RegOrLit1::lit(bytes[param]))
					} else {
						Ok(RegOrLit1::reg(_decode_reg(param)?))
					}
				},
				move |param: usize| -> Result<RegOrLit2, InstrDecodingError> {
					if matches!(bytes[0] & (1 << (3 - param)), 0) {
						Ok(RegOrLit2::lit(u16::from_be_bytes([
							bytes[param],
							bytes[param + 1],
						])))
					} else {
						Ok(RegOrLit2::reg(_decode_reg(param)?))
					}
				},
			)
		};

		match opcode {
			0x01 => Ok(Self::Cpy(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x02 => Ok(Self::Ex(arg_reg(1)?, arg_reg(2)?)),
			0x03 => Ok(Self::Add(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x04 => Ok(Self::Sub(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x05 => Ok(Self::Mul(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x06 => Ok(Self::Div(
				arg_reg(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x07 => Ok(Self::Mod(
				arg_reg(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x08 => Ok(Self::And(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x09 => Ok(Self::Bor(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x0A => Ok(Self::Xor(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x0B => Ok(Self::Shl(arg_reg(1)?, arg_reg_or_lit_1(2)?)),
			0x0C => Ok(Self::Shr(arg_reg(1)?, arg_reg_or_lit_1(2)?)),
			0x0D => Ok(Self::Cmp(arg_reg(1)?, arg_reg_or_lit_2(2)?)),
			0x0E => Ok(Self::Jpr(arg_reg_or_lit_2(1)?)),
			0x0F => Ok(Self::Lsm(arg_reg_or_lit_2(1)?)),
			0x10 => Ok(Self::Itr(arg_reg_or_lit_1(1)?)),
			0x11 => Ok(Self::If(arg_reg_or_lit_1(1)?)),
			0x12 => Ok(Self::IfN(arg_reg_or_lit_1(1)?)),
			0x13 => Ok(Self::If2(
				arg_reg_or_lit_1(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x14 => Ok(Self::Lsa(
				arg_reg(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x15 => Ok(Self::Lea(
				arg_reg_or_lit_1(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x16 => Ok(Self::Wsa(
				arg_reg_or_lit_1(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x17 => Ok(Self::Wea(
				arg_reg_or_lit_1(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x18 => Ok(Self::Srm(
				arg_reg_or_lit_1(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg(3)?,
			)),
			0x19 => Ok(Self::Push(arg_reg_or_lit_2(1)?)),
			0x1A => Ok(Self::Pop(arg_reg(1)?)),
			0x1B => Ok(Self::Call(arg_reg_or_lit_2(1)?)),
			0x1C => Ok(Self::Hwd(
				arg_reg(1)?,
				arg_reg_or_lit_1(2)?,
				arg_reg_or_lit_1(3)?,
			)),
			0x1D => Ok(Self::Cycles(arg_reg(1)?)),
			0x1E => Ok(Self::Halt),
			0x1F => Ok(Self::Reset(arg_reg_or_lit_1(1)?)),
			_ => Err(InstrDecodingError::UnknownOpCode { opcode }),
		}
	}

	#[must_use]
	pub fn encode(self) -> [u8; 4] {
		let mut is_reg = Vec::<bool>::new();
		let mut params = Vec::<u8>::new();

		macro_rules! regs {
            ($($is_reg:expr),*) => {{
                is_reg = vec![$($is_reg),*];
            }};
        }

		macro_rules! push {
			(regs $($reg:expr),*) => {{
                $(params.push($reg.code()));*
            }};
            (regs_or_lit $($val:expr),*) => {{
                $(params.extend_from_slice(&$val.value().to_be_bytes()));*
            }};
		}

		let opcode = match self {
			Self::Cpy(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x01
			}
			Self::Ex(a, b) => {
				regs!(true, true);
				push!(regs a, b);
				0x02
			}
			Self::Add(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x03
			}
			Self::Sub(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x04
			}
			Self::Mul(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x05
			}
			Self::Div(a, b, c) => {
				regs!(true, b.is_reg(), c.is_reg());
				push!(regs a);
				push!(regs_or_lit b, c);
				0x06
			}
			Self::Mod(a, b, c) => {
				regs!(true, b.is_reg(), c.is_reg());
				push!(regs a);
				push!(regs_or_lit b, c);
				0x07
			}
			Self::And(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x08
			}
			Self::Bor(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x09
			}
			Self::Xor(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x0A
			}
			Self::Shl(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x0B
			}
			Self::Shr(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x0C
			}
			Self::Cmp(a, b) => {
				regs!(true, b.is_reg());
				push!(regs a);
				push!(regs_or_lit b);
				0x0D
			}
			Self::Jpr(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x0E
			}
			Self::Lsm(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x0F
			}
			Self::Itr(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x10
			}
			Self::If(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x11
			}
			Self::IfN(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x12
			}
			Self::If2(a, b, c) => {
				regs!(a.is_reg(), b.is_reg(), c.is_reg());
				push!(regs_or_lit a, b, c);
				0x13
			}
			Self::Lsa(a, b, c) => {
				regs!(true, b.is_reg(), c.is_reg());
				push!(regs a);
				push!(regs_or_lit b, c);
				0x14
			}
			Self::Lea(a, b, c) => {
				regs!(a.is_reg(), b.is_reg(), c.is_reg());
				push!(regs_or_lit a, b, c);
				0x15
			}
			Self::Wsa(a, b, c) => {
				regs!(a.is_reg(), b.is_reg(), c.is_reg());
				push!(regs_or_lit a, b, c);
				0x16
			}
			Self::Wea(a, b, c) => {
				regs!(a.is_reg(), b.is_reg(), c.is_reg());
				push!(regs_or_lit a, b, c);
				0x17
			}
			Self::Srm(a, b, c) => {
				regs!(a.is_reg(), b.is_reg(), true);
				push!(regs_or_lit a, b);
				push!(regs c);
				0x18
			}
			Self::Push(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x19
			}
			Self::Pop(a) => {
				regs!(true);
				push!(regs a);
				0x1A
			}
			Self::Call(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x1B
			}
			Self::Hwd(a, b, c) => {
				regs!(true, b.is_reg(), c.is_reg());
				push!(regs a);
				push!(regs_or_lit b, c);
				0x1C
			}
			Self::Cycles(a) => {
				regs!(true);
				push!(regs a);
				0x1D
			}
			Self::Halt => 0x1E,
			Self::Reset(a) => {
				regs!(a.is_reg());
				push!(regs_or_lit a);
				0x1F
			}
		};

		assert!(
			is_reg.len() <= 3,
			"internal error: more than 3 serialized parameters"
		);

		assert!(
			params.len() <= 3,
			"internal error: serialized parameters length exceeds 3 bytes"
		);

		is_reg.resize(3, false);
		params.resize(3, 0);

		[
			(opcode << 3)
				+ if is_reg[0] { 1 << 2 } else { 0 }
				+ if is_reg[1] { 1 << 1 } else { 0 }
				+ u8::from(is_reg[2]),
			params[0],
			params[1],
			params[2],
		]
	}

	#[must_use]
	pub fn encode_word(self) -> u32 {
		u32::from_be_bytes(self.encode())
	}
}

impl ToLasm for Instr {
	fn to_lasm(&self) -> Cow<'static, str> {
		Cow::Owned(match *self {
			Self::Cpy(Reg::Pc, b) => format!("jp {}", b.to_lasm()),
			Self::Cpy(a, RegOrLit2::Lit(0))
			| Self::Mul(a, RegOrLit2::Lit(0))
			| Self::And(a, RegOrLit2::Lit(0))
			| Self::Xor(_, RegOrLit2::Reg(a)) => {
				format!("zro {}", a.to_lasm())
			}
			Self::Cpy(a, b) => format!("cpy {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Ex(a, b) => format!("ex {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Add(a, RegOrLit2::Lit(1)) => format!("inc {}", a.to_lasm()),
			Self::Add(a, b) => format!("add {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Sub(a, RegOrLit2::Lit(1)) => format!("dec {}", a.to_lasm()),
			Self::Sub(a, b) => format!("sub {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Mul(a, b) => format!("mul {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Div(a, b, RegOrLit1::Reg(c)) => {
				format!("div {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm())
			}
			Self::Mod(a, b, RegOrLit1::Reg(c)) => {
				format!("mod {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm())
			}
			Self::Div(a, b, RegOrLit1::Lit(mode)) | Self::Mod(a, b, RegOrLit1::Lit(mode)) => {
				format!(
					"{} {}, {}, {}",
					if matches!(self, Self::Div(_, _, _)) {
						"div"
					} else {
						"mod"
					},
					a.to_lasm(),
					b.to_lasm(),
					DivMode::decode(mode).map_or_else(
						|| format!("{mode:#010b} ; warning: invalid division mode"),
						|mode| mode.to_lasm().into_owned()
					)
				)
			}
			Self::And(a, b) => format!("and {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Bor(a, b) => format!("bor {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Xor(a, b) => format!("xor {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Shl(a, b) => format!("shl {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Shr(a, b) => format!("shr {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Cmp(a, b) => format!("cmp {}, {}", a.to_lasm(), b.to_lasm()),
			Self::Jpr(a) => format!("jpr {}", a.to_lasm_signed()),
			Self::Lsm(a) => format!("lsm {}", a.to_lasm()),
			Self::Itr(a) => format!("itr {}", a.to_lasm()),
			Self::If(RegOrLit1::Reg(reg)) => format!("if {}", reg.to_lasm()),
			Self::If(RegOrLit1::Lit(lit)) => match ArFlag::decode(lit) {
				Some(ArFlag::Zero) => "ifeq".to_owned(),
				Some(ArFlag::Carry) => "ifls".to_owned(),
				Some(flag) => format!("if {}", flag.to_lasm()),
				None => format!("if {lit:#X} ; warning: unknown flag"),
			},
			Self::IfN(RegOrLit1::Reg(reg)) => format!("ifn {}", reg.to_lasm()),
			Self::IfN(RegOrLit1::Lit(lit)) => match ArFlag::decode(lit) {
				Some(ArFlag::Zero) => "ifnq".to_owned(),
				Some(ArFlag::Carry) => "ifge".to_owned(),
				Some(flag) => format!("ifn {}", flag.to_lasm()),
				None => format!("ifn {lit:#X} ; warning: unknown flag"),
			},
			Self::If2(a, b, c) => {
				enum Pos {
					Left,
					Right,
				}

				let mut warns = Vec::new();

				let mut decode_flag = |flag: RegOrLit1, pos| {
					flag.to_lasm_with(|lit| {
						ArFlag::decode(lit).map_or_else(
							|| {
								warns.push(match pos {
									Pos::Left => "invalid left flag",
									Pos::Right => "invalid right flag",
								});
								format!("{lit:#004X}")
							},
							|lit| lit.to_lasm().into_owned(),
						)
					})
				};

				let no_warn = match (a, b, c) {
					(_, _, RegOrLit1::Lit(cond)) => match If2Cond::decode(cond) {
						Some(If2Cond::Nor)
							if matches!(
								(a, b),
								(RegOrLit1::Lit(cst::ZF), RegOrLit1::Lit(cst::CF))
							) =>
						{
							"ifgt".to_owned()
						}
						Some(If2Cond::Or)
							if matches!(
								(a, b),
								(RegOrLit1::Lit(cst::ZF), RegOrLit1::Lit(cst::CF))
							) =>
						{
							"ifle".to_owned()
						}
						Some(cond) => format!(
							"{} {}, {}",
							match cond {
								If2Cond::Or => "ifor",
								If2Cond::And => "ifand",
								If2Cond::Xor => "ifxor",
								If2Cond::Nor => "ifnor",
								If2Cond::Nand => "ifnand",
								If2Cond::Left => "ifleft",
								If2Cond::Right => "ifright",
							},
							decode_flag(a, Pos::Left),
							decode_flag(b, Pos::Right)
						),
						None => {
							let a = decode_flag(a, Pos::Left);
							let b = decode_flag(b, Pos::Right);
							warns.push("invalid condition");
							format!("if2 {a}, {b}, {cond:#004X}")
						}
					},
					(RegOrLit1::Reg(a), RegOrLit1::Reg(b), RegOrLit1::Reg(c)) => {
						format!("if2 {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm())
					}
					(_, _, RegOrLit1::Reg(cond)) => format!(
						"if2 {}, {}, {}",
						decode_flag(a, Pos::Left),
						decode_flag(b, Pos::Right),
						cond.to_lasm()
					),
				};

				if warns.is_empty() {
					no_warn
				} else {
					format!("{no_warn} ; {}", warns.join(", "))
				}
			}
			Self::Lsa(a, b, c) => format!("lsa {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm()),
			Self::Lea(a, b, c) => format!("lea {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm()),
			Self::Wsa(a, b, c) => format!("wsa {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm()),
			Self::Wea(a, b, c) => format!("wea {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm()),
			Self::Srm(a, b, c) => format!("srm {}, {}, {}", a.to_lasm(), b.to_lasm(), c.to_lasm()),
			Self::Push(a) => format!("push {}", a.to_lasm()),
			Self::Pop(Reg::Pc) => "ret".to_owned(),
			Self::Pop(a) => format!("pop {}", a.to_lasm()),
			Self::Call(a) => format!("call {}", a.to_lasm()),
			Self::Hwd(a, b, c) => format!(
				"hwd {}, {}, {}",
				a.to_lasm(),
				b.to_lasm(),
				c.to_lasm_with(|lit| HwInfo::decode(lit).map_or_else(
					|| format!("{lit:#X} ; warning: unknown hardware information"),
					|info| info.to_lasm().into_owned()
				))
			),
			Self::Cycles(a) => format!("cycles {}", a.to_lasm()),
			Self::Halt => return Cow::Borrowed("halt"),
			Self::Reset(a) => format!("reset {}", a.to_lasm()),
		})
	}
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum InstrDecodingError {
	#[error("the provided program's length is not a multiple of 4 bytes (unaligned instructions)")]
	SourceNotMultipleBytesOf4Bytes,
	#[error("unknown opcode: {opcode:#004X}")]
	UnknownOpCode { opcode: u8 },
	#[error("parameter {} uses unknown register: {code:#004X}", .param + 1)]
	UnknownRegister { param: usize, code: u8 },
}
