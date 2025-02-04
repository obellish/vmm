use alloc::string::String;
use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::Deref,
};

use vmm_core::Felt;

#[derive(Debug, Clone)]
pub enum DocumentationType {
	Module(String),
	Form(String),
}

impl Deref for DocumentationType {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Module(s) | Self::Form(s) => s,
		}
	}
}

impl From<DocumentationType> for String {
	fn from(value: DocumentationType) -> Self {
		match value {
			DocumentationType::Form(s) | DocumentationType::Module(s) => s,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexEncodedValue {
	U8(u8),
	U16(u16),
	U32(u32),
	Felt(Felt),
	Word([Felt; 4]),
}

impl Display for HexEncodedValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::U8(value) => Display::fmt(&value, f),
			Self::U16(value) => Display::fmt(&value, f),
			Self::U32(value) => Display::fmt(&value, f),
			Self::Felt(value) => write!(f, "{:#08x}", &value.as_int().to_be()),
			Self::Word(value) => write!(
				f,
				"{:#08x}{:08x}{:08x}{:08x}",
				&value[0].as_int(),
				&value[1].as_int(),
				&value[2].as_int(),
				&value[3].as_int()
			),
		}
	}
}

impl Hash for HexEncodedValue {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			Self::U8(value) => value.hash(state),
			Self::U16(value) => value.hash(state),
			Self::U32(value) => value.hash(state),
			Self::Felt(value) => value.as_int().hash(state),
			Self::Word(word) => {
				word.map(|v| v.as_int()).hash(state);
			}
		}
	}
}

impl Ord for HexEncodedValue {
	#[allow(clippy::match_same_arms)]
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(Self::U8(l), Self::U8(r)) => l.cmp(r),
			(Self::U8(_), _) => Ordering::Less,
			(Self::U16(_), Self::U8(_)) => Ordering::Greater,
			(Self::U16(l), Self::U16(r)) => l.cmp(r),
			(Self::U16(_), _) => Ordering::Less,
			(Self::U32(_), Self::U8(_) | Self::U16(_)) => Ordering::Greater,
			(Self::U32(l), Self::U32(r)) => l.cmp(r),
			(Self::U32(_), _) => Ordering::Greater,
			(Self::Felt(_), Self::U8(_) | Self::U16(_) | Self::U32(_)) => Ordering::Greater,
			(Self::Felt(l), Self::Felt(r)) => l.as_int().cmp(&r.as_int()),
			(Self::Felt(_), _) => Ordering::Greater,
			(Self::Word([l0, l1, l2, l3]), Self::Word([r0, r1, r2, r3])) => l0
				.as_int()
				.cmp(&r0.as_int())
				.then_with(|| l1.as_int().cmp(&r1.as_int()))
				.then_with(|| l2.as_int().cmp(&r2.as_int()))
				.then_with(|| l3.as_int().cmp(&r3.as_int())),
			(Self::Word(_), _) => Ordering::Greater,
		}
	}
}

impl PartialOrd for HexEncodedValue {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinEncodedValue {
	U8(u8),
	U16(u16),
	U32(u32),
}

#[derive(Debug, Clone)]
pub enum Token<'input> {
	Add,
	Adv,
	InsertHdword,
	InsertHdwordWithDomain,
	InsertHperm,
	InsertMem,
	AdvLoadw,
	AdvPipe,
	AdvPush,
	PushExt2intt,
	PushMapval,
	PushMapvaln,
	PushMtnode,
	PushSig,
	PushSmtpeek,
	PushSmtset,
	PushSmtget,
	PushU64Div,
	And,
	Assert,
	Assertz,
	AssertEq,
	AssertEqw,
	Begin,
	Caller,
	Call,
	Cdrop,
	Cdropw,
	Clk,
	Const,
	Cswap,
	Cswapw,
	Debug,
	Div,
	Drop,
	Dropw,
	Dup,
	Dupw,
	DynExec,
	DynCall,
	Else,
	Emit,
	End,
	Eq,
	Eqw,
	Ext2Add,
	Ext2Div,
	Ext2Inv,
	Ext2Mul,
	Ext2Neg,
	Ext2Sub,
	Err,
	Exec,
	Export,
	Exp,
	ExpU,
	False,
	FriExt2Fold4,
	Gt,
	Gte,
	Hash,
	Hperm,
	Hmerge,
	If,
	ILog2,
	Inv,
	IsOdd,
	Local,
	Locaddr,
	LocLoad,
	LocLoadw,
	LocStore,
	LocStorew,
	Lt,
	Lte,
	Mem,
	MemLoad,
	MemLoadw,
	MemStore,
	MemStorew,
	MemStream,
	Movdn,
	Movdnw,
	Movup,
	Movupw,
	MtreeGet,
	MtreeMerge,
	MtreeSet,
	MtreeVerify,
	Mul,
	Neg,
	Neq,
	Not,
	Nop,
	Or,
	Padw,
	Pow2,
	Proc,
	Procref,
	Push,
	RCombBase,
	Repeat,
	RpoFalcon512,
	Sdepth,
	Stack,
	Sub,
	Swap,
	Swapw,
	Swapdw,
	SysCall,
	Trace,
	True,
	Use,
	U32And,
	U32Assert,
	U32Assert2,
	U32Assertw,
	U32Cast,
	U32Div,
	U32Divmod,
	U32Gt,
	U32Gte,
	U32Lt,
	U32Lte,
	U32Max,
	U32Min,
	U32Mod,
	U32Not,
	U32Or,
	U32OverflowingAdd,
	U32OverflowingAdd3,
	U32OverflowingMadd,
	U32OverflowingMul,
	U32OverflowingSub,
	U32Popcnt,
	U32Clz,
	U32Ctz,
	U32Clo,
	U32Cto,
	U32Rotl,
	U32Rotr,
	U32Shl,
	U32Shr,
	U32Split,
	U32Test,
	U32Testw,
	U32WrappingAdd,
	U32WrappingAdd3,
	U32WrappingMadd,
	U32WrappingMul,
	U32WrappingSub,
	U32Xor,
	While,
	Xor,
	At,
	Bang,
	ColonColon,
	Dot,
	Comma,
	Equal,
	Lparen,
	Lbracket,
	Minus,
	Plus,
	SlashSlash,
	Slash,
	Star,
	Rparen,
	Rbracket,
	Rstab,
	DocComment(DocumentationType),
	HexValue(HexEncodedValue),
	BinValue(BinEncodedValue),
	Int(u64),
	Ident(&'input str),
	ConstantIdent(&'input str),
	QuotedIdent(&'input str),
	QuotedString(&'input str),
	Comment,
	Eof,
}
