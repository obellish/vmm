mod decorators;

#[rustfmt::skip]
pub(super) mod opcode_constants {
    pub const OPCODE_NOOP: u8       = 0b0000_0000;
    pub const OPCODE_EQZ: u8        = 0b0000_0001;
    pub const OPCODE_NEG: u8        = 0b0000_0010;
    pub const OPCODE_INV: u8        = 0b0000_0011;
    pub const OPCODE_INCR: u8       = 0b0000_0100;
    pub const OPCODE_NOT: u8        = 0b0000_0101;
    pub const OPCODE_FMPADD: u8     = 0b0000_0110;
    pub const OPCODE_MLOAD: u8      = 0b0000_0111;
    pub const OPCODE_SWAP: u8       = 0b0000_1000;
    pub const OPCODE_CALLER: u8     = 0b0000_1001;
    pub const OPCODE_MOVUP2: u8     = 0b0000_1010;
    pub const OPCODE_MOVDN2: u8     = 0b0000_1011;
    pub const OPCODE_MOVUP3: u8     = 0b0000_1100;
    pub const OPCODE_MOVDN3: u8     = 0b0000_1101;
    pub const OPCODE_ADVPOPW: u8    = 0b0000_1110;
    pub const OPCODE_EXPACC: u8     = 0b0000_1111;

    pub const OPCODE_MOVUP4: u8     = 0b0001_0000;
    pub const OPCODE_MOVDN4: u8     = 0b0001_0001;
    pub const OPCODE_MOVUP5: u8     = 0b0001_0010;
    pub const OPCODE_MOVDN5: u8     = 0b0001_0011;
    pub const OPCODE_MOVUP6: u8     = 0b0001_0100;
    pub const OPCODE_MOVDN6: u8     = 0b0001_0101;
    pub const OPCODE_MOVUP7: u8     = 0b0001_0110;
    pub const OPCODE_MOVDN7: u8     = 0b0001_0111;
    pub const OPCODE_SWAPW: u8      = 0b0001_1000;
    pub const OPCODE_EXT2MUL: u8    = 0b0001_1001;
    pub const OPCODE_MOVUP8: u8     = 0b0001_1010;
    pub const OPCODE_MOVDN8: u8     = 0b0001_1011;
    pub const OPCODE_SWAPW2: u8     = 0b0001_1100;
    pub const OPCODE_SWAPW3: u8     = 0b0001_1101;
    pub const OPCODE_SWAPDW: u8     = 0b0001_1110;

    pub const OPCODE_ASSERT: u8     = 0b0010_0000;
    pub const OPCODE_EQ: u8         = 0b0010_0001;
    pub const OPCODE_ADD: u8        = 0b0010_0010;
    pub const OPCODE_MUL: u8        = 0b0010_0011;
    pub const OPCODE_AND: u8        = 0b0010_0100;
    pub const OPCODE_OR: u8         = 0b0010_0101;
    pub const OPCODE_U32AND: u8     = 0b0010_0110;
    pub const OPCODE_U32XOR: u8     = 0b0010_0111;
    pub const OPCODE_FRIE2F4: u8    = 0b0010_1000;
    pub const OPCODE_DROP: u8       = 0b0010_1001;
    pub const OPCODE_CSWAP: u8      = 0b0010_1010;
    pub const OPCODE_CSWAPW: u8     = 0b0010_1011;
    pub const OPCODE_MLOADW: u8     = 0b0010_1100;
    pub const OPCODE_MSTORE: u8     = 0b0010_1101;
    pub const OPCODE_MSTOREW: u8    = 0b0010_1110;
    pub const OPCODE_FMPUPDATE: u8  = 0b0010_1111;

    pub const OPCODE_PAD: u8        = 0b0011_0000;
    pub const OPCODE_DUP0: u8       = 0b0011_0001;
    pub const OPCODE_DUP1: u8       = 0b0011_0010;
    pub const OPCODE_DUP2: u8       = 0b0011_0011;
    pub const OPCODE_DUP3: u8       = 0b0011_0100;
    pub const OPCODE_DUP4: u8       = 0b0011_0101;
    pub const OPCODE_DUP5: u8       = 0b0011_0110;
    pub const OPCODE_DUP6: u8       = 0b0011_0111;
    pub const OPCODE_DUP7: u8       = 0b0011_1000;
    pub const OPCODE_DUP9: u8       = 0b0011_1001;
    pub const OPCODE_DUP11: u8      = 0b0011_1010;
    pub const OPCODE_DUP13: u8      = 0b0011_1011;
    pub const OPCODE_DUP15: u8      = 0b0011_1100;
    pub const OPCODE_ADVPOP: u8     = 0b0011_1101;
    pub const OPCODE_SDEPTH: u8     = 0b0011_1110;
    pub const OPCODE_CLK: u8        = 0b0011_1111;

    pub const OPCODE_U32ADD: u8     = 0b0100_0000;
    pub const OPCODE_U32SUB: u8     = 0b0100_0010;
    pub const OPCODE_U32MUL: u8     = 0b0100_0100;
    pub const OPCODE_U32DIV: u8     = 0b0100_0110;
    pub const OPCODE_U32SPLIT: u8   = 0b0100_1000;
    pub const OPCODE_U32ASSERT2: u8 = 0b0100_1010;
    pub const OPCODE_U32ADD3: u8    = 0b0100_1100;
    pub const OPCODE_U32MADD: u8    = 0b0100_1110;

    pub const OPCODE_HPERM: u8      = 0b0101_0000;
    pub const OPCODE_MPVERIFY: u8   = 0b0101_0001;
    pub const OPCODE_PIPE: u8       = 0b0101_0010;
    pub const OPCODE_MSTREAM: u8    = 0b0101_0011;
    pub const OPCODE_SPLIT: u8      = 0b0101_0100;
    pub const OPCODE_LOOP: u8       = 0b0101_0101;
    pub const OPCODE_SPAN: u8       = 0b0101_0110;
    pub const OPCODE_JOIN: u8       = 0b0101_0111;
    pub const OPCODE_DYN: u8        = 0b0101_1000;
    pub const OPCODE_RCOMBBASE: u8  = 0b0101_1001;
    pub const OPCODE_EMIT: u8       = 0b0101_1010;
    pub const OPCODE_PUSH: u8       = 0b0101_1011;
    pub const OPCODE_DYNCALL: u8    = 0b0101_1100;

    pub const OPCODE_MRUPDATE: u8   = 0b0110_0000;
    /* unused:                        0b0110_0100 */
    pub const OPCODE_SYSCALL: u8    = 0b0110_1000;
    pub const OPCODE_CALL: u8       = 0b0110_1100;
    pub const OPCODE_END: u8        = 0b0111_0000;
    pub const OPCODE_REPEAT: u8     = 0b0111_0100;
    pub const OPCODE_RESPAN: u8     = 0b0111_1000;
    pub const OPCODE_HALT: u8       = 0b0111_1100;
}

use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

pub use self::decorators::{
	AssemblyOp, DebugOptions, Decorator, DecoratorIterator, DecoratorList, DecoratorSlice,
	SignatureKind,
};
use self::opcode_constants::{
	OPCODE_ADD, OPCODE_ADVPOP, OPCODE_ADVPOPW, OPCODE_AND, OPCODE_ASSERT, OPCODE_CALL,
	OPCODE_CALLER, OPCODE_CLK, OPCODE_CSWAP, OPCODE_CSWAPW, OPCODE_DROP, OPCODE_DUP0, OPCODE_DUP1,
	OPCODE_DUP2, OPCODE_DUP3, OPCODE_DUP4, OPCODE_DUP5, OPCODE_DUP6, OPCODE_DUP7, OPCODE_DUP9,
	OPCODE_DUP11, OPCODE_DUP13, OPCODE_DUP15, OPCODE_DYN, OPCODE_DYNCALL, OPCODE_EMIT, OPCODE_END,
	OPCODE_EQ, OPCODE_EQZ, OPCODE_EXPACC, OPCODE_EXT2MUL, OPCODE_FMPADD, OPCODE_FMPUPDATE,
	OPCODE_FRIE2F4, OPCODE_HALT, OPCODE_HPERM, OPCODE_INCR, OPCODE_INV, OPCODE_JOIN, OPCODE_LOOP,
	OPCODE_MLOAD, OPCODE_MLOADW, OPCODE_MOVDN2, OPCODE_MOVDN3, OPCODE_MOVDN4, OPCODE_MOVDN5,
	OPCODE_MOVDN6, OPCODE_MOVDN7, OPCODE_MOVDN8, OPCODE_MOVUP2, OPCODE_MOVUP3, OPCODE_MOVUP4,
	OPCODE_MOVUP5, OPCODE_MOVUP6, OPCODE_MOVUP7, OPCODE_MOVUP8, OPCODE_MPVERIFY, OPCODE_MRUPDATE,
	OPCODE_MSTORE, OPCODE_MSTOREW, OPCODE_MSTREAM, OPCODE_MUL, OPCODE_NEG, OPCODE_NOOP, OPCODE_NOT,
	OPCODE_OR, OPCODE_PAD, OPCODE_PIPE, OPCODE_PUSH, OPCODE_RCOMBBASE, OPCODE_REPEAT,
	OPCODE_RESPAN, OPCODE_SDEPTH, OPCODE_SPAN, OPCODE_SPLIT, OPCODE_SWAP, OPCODE_SWAPDW,
	OPCODE_SWAPW, OPCODE_SWAPW2, OPCODE_SWAPW3, OPCODE_SYSCALL, OPCODE_U32ADD, OPCODE_U32ADD3,
	OPCODE_U32AND, OPCODE_U32ASSERT2, OPCODE_U32DIV, OPCODE_U32MADD, OPCODE_U32MUL,
	OPCODE_U32SPLIT, OPCODE_U32SUB, OPCODE_U32XOR,
};
use crate::{
	Felt,
	prettier::{Document, PrettyPrint, display},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Operation {
	Noop = OPCODE_NOOP,
	Assert(u32) = OPCODE_ASSERT,
	FmpAdd = OPCODE_FMPADD,
	FmpUpdate = OPCODE_FMPUPDATE,
	SDepth = OPCODE_SDEPTH,
	Caller = OPCODE_CALLER,
	Clk = OPCODE_CLK,
	Emit(u32) = OPCODE_EMIT,
	Join = OPCODE_JOIN,
	Split = OPCODE_SPLIT,
	Loop = OPCODE_LOOP,
	Call = OPCODE_CALL,
	Dyn = OPCODE_DYN,
	DynCall = OPCODE_DYNCALL,
	SysCall = OPCODE_SYSCALL,
	Span = OPCODE_SPAN,
	End = OPCODE_END,
	Repeat = OPCODE_REPEAT,
	Respan = OPCODE_RESPAN,
	Halt = OPCODE_HALT,
	Add = OPCODE_ADD,
	Neg = OPCODE_NEG,
	Mul = OPCODE_MUL,
	Inv = OPCODE_INV,
	Incr = OPCODE_INCR,
	And = OPCODE_AND,
	Or = OPCODE_OR,
	Not = OPCODE_NOT,
	Eq = OPCODE_EQ,
	Eqz = OPCODE_EQZ,
	Expacc = OPCODE_EXPACC,
	Ext2Mul = OPCODE_EXT2MUL,
	U32split = OPCODE_U32SPLIT,
	U32add = OPCODE_U32ADD,
	U32assert2(u32) = OPCODE_U32ASSERT2,
	U32add3 = OPCODE_U32ADD3,
	U32sub = OPCODE_U32SUB,
	U32mul = OPCODE_U32MUL,
	U32madd = OPCODE_U32MADD,
	U32div = OPCODE_U32DIV,
	U32and = OPCODE_U32AND,
	U32xor = OPCODE_U32XOR,
	Pad = OPCODE_PAD,
	Drop = OPCODE_DROP,
	Dup0 = OPCODE_DUP0,
	Dup1 = OPCODE_DUP1,
	Dup2 = OPCODE_DUP2,
	Dup3 = OPCODE_DUP3,
	Dup4 = OPCODE_DUP4,
	Dup5 = OPCODE_DUP5,
	Dup6 = OPCODE_DUP6,
	Dup7 = OPCODE_DUP7,
	Dup9 = OPCODE_DUP9,
	Dup11 = OPCODE_DUP11,
	Dup13 = OPCODE_DUP13,
	Dup15 = OPCODE_DUP15,
	Swap = OPCODE_SWAP,
	SwapW = OPCODE_SWAPW,
	SwapW2 = OPCODE_SWAPW2,
	SwapW3 = OPCODE_SWAPW3,
	SwapDW = OPCODE_SWAPDW,
	MovUp2 = OPCODE_MOVUP2,
	MovUp3 = OPCODE_MOVUP3,
	MovUp4 = OPCODE_MOVUP4,
	MovUp5 = OPCODE_MOVUP5,
	MovUp6 = OPCODE_MOVUP6,
	MovUp7 = OPCODE_MOVUP7,
	MovUp8 = OPCODE_MOVUP8,
	MovDn2 = OPCODE_MOVDN2,
	MovDn3 = OPCODE_MOVDN3,
	MovDn4 = OPCODE_MOVDN4,
	MovDn5 = OPCODE_MOVDN5,
	MovDn6 = OPCODE_MOVDN6,
	MovDn7 = OPCODE_MOVDN7,
	MovDn8 = OPCODE_MOVDN8,
	CSwap = OPCODE_CSWAP,
	CSwapW = OPCODE_CSWAPW,
	Push(Felt) = OPCODE_PUSH,
	AdvPop = OPCODE_ADVPOP,
	AdvPopW = OPCODE_ADVPOPW,
	MLoadW = OPCODE_MLOADW,
	MStoreW = OPCODE_MSTOREW,
	MLoad = OPCODE_MLOAD,
	MStore = OPCODE_MSTORE,
	MStream = OPCODE_MSTREAM,
	Pipe = OPCODE_PIPE,
	HPerm = OPCODE_HPERM,
	MpVerify(u32) = OPCODE_MPVERIFY,
	MrUpdate = OPCODE_MRUPDATE,
	FriE2F4 = OPCODE_FRIE2F4,
	RCombBase = OPCODE_RCOMBBASE,
}

impl Operation {
	pub const OP_BITS: usize = 7;

	#[must_use]
	pub fn op_code(&self) -> u8 {
		unsafe { *<*const _>::from(self).cast::<u8>() }
	}

	#[must_use]
	pub fn imm_value(self) -> Option<Felt> {
		match self {
			Self::Push(imm) => Some(imm),
			Self::Emit(imm) => Some(imm.into()),
			_ => None,
		}
	}

	#[must_use]
	pub const fn populate_decoder_hasher_registers(self) -> bool {
		matches!(
			self,
			Self::End
				| Self::Join | Self::Split
				| Self::Loop | Self::Repeat
				| Self::Respan
				| Self::Span | Self::Halt
				| Self::Call | Self::SysCall
		)
	}
}

impl Deserializable for Operation {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let op_code = source.read_u8()?;

		Ok(match op_code {
			OPCODE_NOOP => Self::Noop,
			OPCODE_EQZ => Self::Eqz,
			OPCODE_NEG => Self::Neg,
			OPCODE_INV => Self::Inv,
			OPCODE_INCR => Self::Incr,
			OPCODE_NOT => Self::Not,
			OPCODE_FMPADD => Self::FmpAdd,
			OPCODE_MLOAD => Self::MLoad,
			OPCODE_SWAP => Self::Swap,
			OPCODE_CALLER => Self::Caller,
			OPCODE_MOVUP2 => Self::MovUp2,
			OPCODE_MOVDN2 => Self::MovDn2,
			OPCODE_MOVUP3 => Self::MovUp3,
			OPCODE_MOVDN3 => Self::MovDn3,
			OPCODE_ADVPOPW => Self::AdvPopW,
			OPCODE_EXPACC => Self::Expacc,
			OPCODE_MOVUP4 => Self::MovUp4,
			OPCODE_MOVDN4 => Self::MovDn4,
			OPCODE_MOVUP5 => Self::MovUp5,
			OPCODE_MOVDN5 => Self::MovDn5,
			OPCODE_MOVUP6 => Self::MovUp6,
			OPCODE_MOVDN6 => Self::MovDn6,
			OPCODE_MOVUP7 => Self::MovUp7,
			OPCODE_MOVDN7 => Self::MovDn7,
			OPCODE_SWAPW => Self::SwapW,
			OPCODE_EXT2MUL => Self::Ext2Mul,
			OPCODE_MOVUP8 => Self::MovUp8,
			OPCODE_MOVDN8 => Self::MovDn8,
			OPCODE_SWAPW2 => Self::SwapW2,
			OPCODE_SWAPW3 => Self::SwapW3,
			OPCODE_SWAPDW => Self::SwapDW,
			OPCODE_ASSERT => {
				let err_code = source.read_u32()?;
				Self::Assert(err_code)
			}
			OPCODE_EQ => Self::Eq,
			OPCODE_ADD => Self::Add,
			OPCODE_MUL => Self::Mul,
			OPCODE_AND => Self::And,
			OPCODE_OR => Self::Or,
			OPCODE_U32AND => Self::U32and,
			OPCODE_U32XOR => Self::U32xor,
			OPCODE_FRIE2F4 => Self::FriE2F4,
			OPCODE_DROP => Self::Drop,
			OPCODE_CSWAP => Self::CSwap,
			OPCODE_CSWAPW => Self::CSwapW,
			OPCODE_MLOADW => Self::MLoadW,
			OPCODE_MSTORE => Self::MStore,
			OPCODE_MSTOREW => Self::MStoreW,
			OPCODE_FMPUPDATE => Self::FmpUpdate,
			OPCODE_PAD => Self::Pad,
			OPCODE_DUP0 => Self::Dup0,
			OPCODE_DUP1 => Self::Dup1,
			OPCODE_DUP2 => Self::Dup2,
			OPCODE_DUP3 => Self::Dup3,
			OPCODE_DUP4 => Self::Dup4,
			OPCODE_DUP5 => Self::Dup5,
			OPCODE_DUP6 => Self::Dup6,
			OPCODE_DUP7 => Self::Dup7,
			OPCODE_DUP9 => Self::Dup9,
			OPCODE_DUP11 => Self::Dup11,
			OPCODE_DUP13 => Self::Dup13,
			OPCODE_DUP15 => Self::Dup15,
			OPCODE_ADVPOP => Self::AdvPop,
			OPCODE_SDEPTH => Self::SDepth,
			OPCODE_CLK => Self::Clk,
			OPCODE_U32ADD => Self::U32add,
			OPCODE_U32SUB => Self::U32sub,
			OPCODE_U32MUL => Self::U32mul,
			OPCODE_U32DIV => Self::U32div,
			OPCODE_U32SPLIT => Self::U32split,
			OPCODE_U32ASSERT2 => {
				let err_code = source.read_u32()?;
				Self::U32assert2(err_code)
			}
			OPCODE_U32ADD3 => Self::U32add3,
			OPCODE_U32MADD => Self::U32madd,
			OPCODE_HPERM => Self::HPerm,
			OPCODE_MPVERIFY => {
				let err_code = source.read_u32()?;
				Self::MpVerify(err_code)
			}
			OPCODE_PIPE => Self::Pipe,
			OPCODE_MSTREAM => Self::MStream,
			OPCODE_SPLIT => Self::Split,
			OPCODE_LOOP => Self::Loop,
			OPCODE_SPAN => Self::Span,
			OPCODE_JOIN => Self::Join,
			OPCODE_DYN => Self::Dyn,
			OPCODE_DYNCALL => Self::DynCall,
			OPCODE_RCOMBBASE => Self::RCombBase,
			OPCODE_MRUPDATE => Self::MrUpdate,
			OPCODE_PUSH => {
				let value_u64 = source.read_u64()?;
				let value_felt = Felt::try_from(value_u64).map_err(|_| {
					DeserializationError::InvalidValue(format!(
						"operation associated data doesn't fit in a field element: {value_u64}"
					))
				})?;
				Self::Push(value_felt)
			}
			OPCODE_EMIT => {
				let value = source.read_u32()?;
				Self::Emit(value)
			}
			OPCODE_SYSCALL => Self::SysCall,
			OPCODE_CALL => Self::Call,
			OPCODE_END => Self::End,
			OPCODE_REPEAT => Self::Repeat,
			OPCODE_RESPAN => Self::Respan,
			OPCODE_HALT => Self::Halt,
			_ => {
				return Err(DeserializationError::InvalidValue(format!(
					"invalid opcode '{op_code}'"
				)));
			}
		})
	}
}

impl Display for Operation {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Noop => f.write_str("noop"),
			Self::Assert(err_code) => {
				f.write_str("assert(")?;
				Display::fmt(&err_code, f)?;
				f.write_char(')')
			}
			Self::FmpAdd => f.write_str("fmpadd"),
			Self::FmpUpdate => f.write_str("fmpupdate"),
			Self::SDepth => f.write_str("sdepth"),
			Self::Caller => f.write_str("caller"),
			Self::Clk => f.write_str("clk"),
			Self::Join => f.write_str("join"),
			Self::Split => f.write_str("split"),
			Self::Loop => f.write_str("loop"),
			Self::Call => f.write_str("call"),
			Self::DynCall => f.write_str("dyncall"),
			Self::SysCall => f.write_str("syscall"),
			Self::Dyn => f.write_str("dyn"),
			Self::Span => f.write_str("span"),
			Self::End => f.write_str("end"),
			Self::Repeat => f.write_str("repeat"),
			Self::Respan => f.write_str("respan"),
			Self::Halt => f.write_str("halt"),
			Self::Add => f.write_str("add"),
			Self::Neg => f.write_str("neg"),
			Self::Mul => f.write_str("mul"),
			Self::Inv => f.write_str("inv"),
			Self::Incr => f.write_str("incr"),
			Self::And => f.write_str("and"),
			Self::Or => f.write_str("or"),
			Self::Not => f.write_str("not"),
			Self::Eq => f.write_str("eq"),
			Self::Eqz => f.write_str("eqz"),
			Self::Expacc => f.write_str("expacc"),
			Self::Ext2Mul => f.write_str("ext2mul"),
			Self::U32assert2(err_code) => {
				f.write_str("u32assert2(")?;
				Display::fmt(&err_code, f)?;
				f.write_char(')')
			}
			Self::U32split => f.write_str("u32split"),
			Self::U32add => f.write_str("u32add"),
			Self::U32add3 => f.write_str("u32add3"),
			Self::U32sub => f.write_str("u32sub"),
			Self::U32mul => f.write_str("u32mul"),
			Self::U32madd => f.write_str("u32madd"),
			Self::U32div => f.write_str("u32div"),
			Self::U32and => f.write_str("u32and"),
			Self::U32xor => f.write_str("u32xor"),
			Self::Drop => f.write_str("drop"),
			Self::Pad => f.write_str("pad"),
			Self::Dup0 => f.write_str("dup0"),
			Self::Dup1 => f.write_str("dup1"),
			Self::Dup2 => f.write_str("dup2"),
			Self::Dup3 => f.write_str("dup3"),
			Self::Dup4 => f.write_str("dup4"),
			Self::Dup5 => f.write_str("dup5"),
			Self::Dup6 => f.write_str("dup6"),
			Self::Dup7 => f.write_str("dup7"),
			Self::Dup9 => f.write_str("dup9"),
			Self::Dup11 => f.write_str("dup11"),
			Self::Dup13 => f.write_str("dup13"),
			Self::Dup15 => f.write_str("dup15"),
			Self::Swap => f.write_str("swap"),
			Self::SwapW => f.write_str("swapw"),
			Self::SwapW2 => f.write_str("swapw2"),
			Self::SwapW3 => f.write_str("swapw3"),
			Self::SwapDW => f.write_str("swapdw"),
			Self::MovUp2 => f.write_str("movup2"),
			Self::MovUp3 => f.write_str("movup3"),
			Self::MovUp4 => f.write_str("movup4"),
			Self::MovUp5 => f.write_str("movup5"),
			Self::MovUp6 => f.write_str("movup6"),
			Self::MovUp7 => f.write_str("movup7"),
			Self::MovUp8 => f.write_str("movup8"),
			Self::MovDn2 => f.write_str("movdn2"),
			Self::MovDn3 => f.write_str("movdn3"),
			Self::MovDn4 => f.write_str("movdn4"),
			Self::MovDn5 => f.write_str("movdn5"),
			Self::MovDn6 => f.write_str("movdn6"),
			Self::MovDn7 => f.write_str("movdn7"),
			Self::MovDn8 => f.write_str("movdn8"),
			Self::CSwap => f.write_str("cswap"),
			Self::CSwapW => f.write_str("cswapw"),
			Self::Push(value) => {
				f.write_str("push(")?;
				Display::fmt(&value, f)?;
				f.write_char(')')
			}
			Self::AdvPop => f.write_str("advpop"),
			Self::AdvPopW => f.write_str("advpopw"),
			Self::MLoadW => f.write_str("mloadw"),
			Self::MStoreW => f.write_str("mstorew"),
			Self::MLoad => f.write_str("mload"),
			Self::MStore => f.write_str("mstore"),
			Self::MStream => f.write_str("mstream"),
			Self::Pipe => f.write_str("pipe"),
			Self::Emit(value) => {
				f.write_str("emit(")?;
				Display::fmt(&value, f)?;
				f.write_char(')')
			}
			Self::HPerm => f.write_str("hperm"),
			Self::MpVerify(err_code) => {
				f.write_str("mpverify(")?;
				Display::fmt(&err_code, f)?;
				f.write_char(')')
			}
			Self::MrUpdate => f.write_str("mrupdate"),
			Self::FriE2F4 => f.write_str("frie2f4"),
			Self::RCombBase => f.write_str("rcomb1"),
		}
	}
}

impl PrettyPrint for Operation {
	fn render(&self) -> Document {
		display(self)
	}
}

impl Serializable for Operation {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u8(self.op_code());

		match self {
			Self::Assert(err_code) | Self::MpVerify(err_code) | Self::U32assert2(err_code) => {
				err_code.write_into(target);
			}
			Self::Push(value) => value.as_int().write_into(target),
			Self::Emit(value) => value.write_into(target),
			_ => {}
		}
	}
}
