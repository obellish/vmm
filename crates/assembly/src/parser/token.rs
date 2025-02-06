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
	PushExt2Intt,
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

impl<'input> Token<'input> {
	const KEYWORDS: &'static [(&'static str, Token<'static>)] = &[
		("add", Token::Add),
		("adv", Token::Adv),
		("insert_hdword", Token::InsertHdword),
		("insert_hdword_d", Token::InsertHdwordWithDomain),
		("insert_hperm", Token::InsertHperm),
		("insert_mem", Token::InsertMem),
		("adv_loadw", Token::AdvLoadw),
		("adv_pipe", Token::AdvPipe),
		("adv_push", Token::AdvPush),
		("push_ext2intt", Token::PushExt2Intt),
		("push_mapval", Token::PushMapval),
		("push_mapvaln", Token::PushMapvaln),
		("push_mtnode", Token::PushMtnode),
		("push_sig", Token::PushSig),
		("push_smtpeek", Token::PushSmtpeek),
		("push_smtset", Token::PushSmtset),
		("push_smtget", Token::PushSmtget),
		("push_u64div", Token::PushU64Div),
		("and", Token::And),
		("assert", Token::Assert),
		("assertz", Token::Assertz),
		("assert_eq", Token::AssertEq),
		("assert_eqw", Token::AssertEqw),
		("begin", Token::Begin),
		("caller", Token::Caller),
		("call", Token::Call),
		("cdrop", Token::Cdrop),
		("cdropw", Token::Cdropw),
		("clk", Token::Clk),
		("const", Token::Const),
		("cswap", Token::Cswap),
		("cswapw", Token::Cswapw),
		("debug", Token::Debug),
		("div", Token::Div),
		("drop", Token::Drop),
		("dropw", Token::Dropw),
		("dup", Token::Dup),
		("dupw", Token::Dupw),
		("dynexec", Token::DynExec),
		("dyncall", Token::DynCall),
		("else", Token::Else),
		("emit", Token::Emit),
		("end", Token::End),
		("eq", Token::Eq),
		("eqw", Token::Eqw),
		("ext2add", Token::Ext2Add),
		("ext2div", Token::Ext2Div),
		("ext2inv", Token::Ext2Inv),
		("ext2mul", Token::Ext2Mul),
		("ext2neg", Token::Ext2Neg),
		("ext2sub", Token::Ext2Sub),
		("err", Token::Err),
		("exec", Token::Exec),
		("exp", Token::Exp),
		("exp.u", Token::ExpU),
		("export", Token::Export),
		("false", Token::False),
		("fri_ext2fold4", Token::FriExt2Fold4),
		("gt", Token::Gt),
		("gte", Token::Gte),
		("hash", Token::Hash),
		("hperm", Token::Hperm),
		("hmerge", Token::Hmerge),
		("if", Token::If),
		("ilog2", Token::ILog2),
		("inv", Token::Inv),
		("is_odd", Token::IsOdd),
		("local", Token::Local),
		("locaddr", Token::Locaddr),
		("loc_load", Token::LocLoad),
		("loc_loadw", Token::LocLoadw),
		("loc_store", Token::LocStore),
		("loc_storew", Token::LocStorew),
		("lt", Token::Lt),
		("lte", Token::Lte),
		("mem", Token::Mem),
		("mem_load", Token::MemLoad),
		("mem_loadw", Token::MemLoadw),
		("mem_store", Token::MemStore),
		("mem_storew", Token::MemStorew),
		("mem_stream", Token::MemStream),
		("movdn", Token::Movdn),
		("movdnw", Token::Movdnw),
		("movup", Token::Movup),
		("movupw", Token::Movupw),
		("mtree_get", Token::MtreeGet),
		("mtree_merge", Token::MtreeMerge),
		("mtree_set", Token::MtreeSet),
		("mtree_verify", Token::MtreeVerify),
		("mul", Token::Mul),
		("neg", Token::Neg),
		("neq", Token::Neq),
		("not", Token::Not),
		("nop", Token::Nop),
		("or", Token::Or),
		("padw", Token::Padw),
		("pow2", Token::Pow2),
		("proc", Token::Proc),
		("procref", Token::Procref),
		("push", Token::Push),
		("rcomb_base", Token::RCombBase),
		("repeat", Token::Repeat),
		("rpo_falcon512", Token::RpoFalcon512),
		("sdepth", Token::Sdepth),
		("stack", Token::Stack),
		("sub", Token::Sub),
		("swap", Token::Swap),
		("swapw", Token::Swapw),
		("swapdw", Token::Swapdw),
		("syscall", Token::SysCall),
		("trace", Token::Trace),
		("true", Token::True),
		("use", Token::Use),
		("u32and", Token::U32And),
		("u32assert", Token::U32Assert),
		("u32assert2", Token::U32Assert2),
		("u32assertw", Token::U32Assertw),
		("u32cast", Token::U32Cast),
		("u32div", Token::U32Div),
		("u32divmod", Token::U32Divmod),
		("u32gt", Token::U32Gt),
		("u32gte", Token::U32Gte),
		("u32lt", Token::U32Lt),
		("u32lte", Token::U32Lte),
		("u32max", Token::U32Max),
		("u32min", Token::U32Min),
		("u32mod", Token::U32Mod),
		("u32not", Token::U32Not),
		("u32or", Token::U32Or),
		("u32overflowing_add", Token::U32OverflowingAdd),
		("u32overflowing_add3", Token::U32OverflowingAdd3),
		("u32overflowing_madd", Token::U32OverflowingMadd),
		("u32overflowing_mul", Token::U32OverflowingMul),
		("u32overflowing_sub", Token::U32OverflowingSub),
		("u32popcnt", Token::U32Popcnt),
		("u32clz", Token::U32Clz),
		("u32ctz", Token::U32Ctz),
		("u32clo", Token::U32Clo),
		("u32cto", Token::U32Cto),
		("u32rotl", Token::U32Rotl),
		("u32rotr", Token::U32Rotr),
		("u32shl", Token::U32Shl),
		("u32shr", Token::U32Shr),
		("u32split", Token::U32Split),
		("u32test", Token::U32Test),
		("u32testw", Token::U32Testw),
		("u32wrapping_add", Token::U32WrappingAdd),
		("u32wrapping_add3", Token::U32WrappingAdd3),
		("u32wrapping_madd", Token::U32WrappingMadd),
		("u32wrapping_mul", Token::U32WrappingMul),
		("u32wrapping_sub", Token::U32WrappingSub),
		("u32xor", Token::U32Xor),
		("while", Token::While),
		("xor", Token::Xor),
	];

	pub const fn is_instruction(&self) -> bool {
		matches!(
			self,
			Self::Add
				| Self::Adv | Self::InsertHdword
				| Self::InsertHdwordWithDomain
				| Self::InsertHperm
				| Self::InsertMem
				| Self::AdvLoadw
				| Self::AdvPipe
				| Self::AdvPush
				| Self::PushExt2Intt
				| Self::PushMapval
				| Self::PushMapvaln
				| Self::PushMtnode
				| Self::PushSig
				| Self::PushSmtpeek
				| Self::PushSmtset
				| Self::PushSmtget
				| Self::PushU64Div
				| Self::And | Self::Assert
				| Self::Assertz
				| Self::AssertEq
				| Self::AssertEqw
				| Self::Caller
				| Self::Call | Self::Cdrop
				| Self::Cdropw
				| Self::Clk | Self::Cswap
				| Self::Cswapw
				| Self::Debug
				| Self::Div | Self::Drop
				| Self::Dropw
				| Self::Dup | Self::Dupw
				| Self::DynExec
				| Self::DynCall
				| Self::Emit | Self::Eq
				| Self::Eqw | Self::Ext2Add
				| Self::Ext2Div
				| Self::Ext2Inv
				| Self::Ext2Mul
				| Self::Ext2Neg
				| Self::Ext2Sub
				| Self::Exec | Self::Exp
				| Self::ExpU | Self::FriExt2Fold4
				| Self::Gt | Self::Gte
				| Self::Hash | Self::Hperm
				| Self::Hmerge
				| Self::ILog2
				| Self::Inv | Self::IsOdd
				| Self::Local
				| Self::Locaddr
				| Self::LocLoad
				| Self::LocLoadw
				| Self::LocStore
				| Self::LocStorew
				| Self::Lt | Self::Lte
				| Self::Mem | Self::MemLoad
				| Self::MemLoadw
				| Self::MemStore
				| Self::MemStorew
				| Self::MemStream
				| Self::Movdn
				| Self::Movdnw
				| Self::Movup
				| Self::Movupw
				| Self::MtreeGet
				| Self::MtreeMerge
				| Self::MtreeSet
				| Self::MtreeVerify
				| Self::Mul | Self::Neg
				| Self::Neq | Self::Not
				| Self::Nop | Self::Or
				| Self::Padw | Self::Pow2
				| Self::Procref
				| Self::Push | Self::RCombBase
				| Self::Repeat
				| Self::Sdepth
				| Self::Stack
				| Self::Sub | Self::Swap
				| Self::Swapw
				| Self::Swapdw
				| Self::SysCall
				| Self::Trace
				| Self::U32And
				| Self::U32Assert
				| Self::U32Assert2
				| Self::U32Assertw
				| Self::U32Cast
				| Self::U32Div
				| Self::U32Divmod
				| Self::U32Gt
				| Self::U32Gte
				| Self::U32Lt
				| Self::U32Lte
				| Self::U32Max
				| Self::U32Min
				| Self::U32Mod
				| Self::U32Not
				| Self::U32Or
				| Self::U32OverflowingAdd
				| Self::U32OverflowingAdd3
				| Self::U32OverflowingMadd
				| Self::U32OverflowingMul
				| Self::U32OverflowingSub
				| Self::U32Popcnt
				| Self::U32Clz
				| Self::U32Ctz
				| Self::U32Clo
				| Self::U32Cto
				| Self::U32Rotl
				| Self::U32Rotr
				| Self::U32Shl
				| Self::U32Shr
				| Self::U32Split
				| Self::U32Test
				| Self::U32Testw
				| Self::U32WrappingAdd
				| Self::U32WrappingAdd3
				| Self::U32WrappingMadd
				| Self::U32WrappingMul
				| Self::U32WrappingSub
				| Self::U32Xor
				| Self::Xor
		)
	}

	pub fn keyword_searcher() -> aho_corasick::AhoCorasick {
		aho_corasick::AhoCorasick::builder()
			.match_kind(aho_corasick::MatchKind::LeftmostLongest)
			.start_kind(aho_corasick::StartKind::Anchored)
			.build(Self::KEYWORDS.iter().map(|(kw, _)| kw).copied())
			.expect("unable to build aho-corasick searcher for token")
	}

	pub fn from_keyword_or_ident(s: &'input str) -> Self {
		let searcher = Self::keyword_searcher();
		Self::from_keyword_or_ident_with_searcher(s, &searcher)
	}

	pub fn from_keyword_or_ident_with_searcher(
		s: &'input str,
		searcher: &aho_corasick::AhoCorasick,
	) -> Self {
		let _ = aho_corasick::Input::new(s).anchored(aho_corasick::Anchored::Yes);
		match searcher.find(s) {
			None => Self::Ident(s),
			Some(matched) if matched.len() != s.len() => Self::Ident(s),
			Some(matched) => Self::KEYWORDS[matched.pattern().as_usize()].1.clone(),
		}
	}

	pub fn parse(s: &'input str) -> Option<Self> {
		match Self::from_keyword_or_ident(s) {
			Self::Ident(_) => Some(match s {
				"@" => Self::At,
				"!" => Self::Bang,
				"::" => Self::ColonColon,
				"." => Self::Dot,
				"," => Self::Comma,
				"=" => Self::Equal,
				"(" => Self::Lparen,
				"[" => Self::Lbracket,
				"-" => Self::Minus,
				"+" => Self::Plus,
				"//" => Self::SlashSlash,
				"/" => Self::Slash,
				"*" => Self::Star,
				")" => Self::Rparen,
				"]" => Self::Rbracket,
				"->" => Self::Rstab,
				"end of file" => Self::Eof,
				"module doc" => Self::DocComment(DocumentationType::Module(String::new())),
				"doc comment" => Self::DocComment(DocumentationType::Form(String::new())),
				"comment" => Self::Comment,
				"hex-encoded value" => Self::HexValue(HexEncodedValue::U8(0)),
				"bin-encoded value" => Self::BinValue(BinEncodedValue::U8(0)),
				"integer" => Self::Int(0),
				"identifier" => Self::Ident(""),
				"constant identifier" => Self::ConstantIdent(""),
				"quoted identifier" => Self::QuotedIdent(""),
				"quoted string" => Self::QuotedString(""),
				_ => return None,
			}),
			token => Some(token),
		}
	}
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Add => write!(f, "add"),
			Self::Adv => write!(f, "adv"),
			Self::InsertHdword => write!(f, "insert_hdword"),
			Self::InsertHdwordWithDomain => write!(f, "insert_hdword_d"),
			Self::InsertHperm => write!(f, "insert_hperm"),
			Self::InsertMem => write!(f, "insert_mem"),
			Self::AdvLoadw => write!(f, "adv_loadw"),
			Self::AdvPipe => write!(f, "adv_pipe"),
			Self::AdvPush => write!(f, "adv_push"),
			Self::PushExt2Intt => write!(f, "push_ext2intt"),
			Self::PushMapval => write!(f, "push_mapval"),
			Self::PushMapvaln => write!(f, "push_mapvaln"),
			Self::PushMtnode => write!(f, "push_mtnode"),
			Self::PushSig => write!(f, "push_sig"),
			Self::PushSmtpeek => write!(f, "push_smtpeek"),
			Self::PushSmtset => write!(f, "push_smtset"),
			Self::PushSmtget => write!(f, "push_smtget"),
			Self::PushU64Div => write!(f, "push_u64div"),
			Self::And => write!(f, "and"),
			Self::Assert => write!(f, "assert"),
			Self::Assertz => write!(f, "assertz"),
			Self::AssertEq => write!(f, "assert_eq"),
			Self::AssertEqw => write!(f, "assert_eqw"),
			Self::Begin => write!(f, "begin"),
			Self::Caller => write!(f, "caller"),
			Self::Call => write!(f, "call"),
			Self::Cdrop => write!(f, "cdrop"),
			Self::Cdropw => write!(f, "cdropw"),
			Self::Clk => write!(f, "clk"),
			Self::Const => write!(f, "const"),
			Self::Cswap => write!(f, "cswap"),
			Self::Cswapw => write!(f, "cswapw"),
			Self::Debug => write!(f, "debug"),
			Self::Div => write!(f, "div"),
			Self::Drop => write!(f, "drop"),
			Self::Dropw => write!(f, "dropw"),
			Self::Dup => write!(f, "dup"),
			Self::Dupw => write!(f, "dupw"),
			Self::DynExec => write!(f, "dynexec"),
			Self::DynCall => write!(f, "dyncall"),
			Self::Else => write!(f, "else"),
			Self::Emit => write!(f, "emit"),
			Self::End => write!(f, "end"),
			Self::Eq => write!(f, "eq"),
			Self::Eqw => write!(f, "eqw"),
			Self::Ext2Add => write!(f, "ext2add"),
			Self::Ext2Div => write!(f, "ext2div"),
			Self::Ext2Inv => write!(f, "ext2inv"),
			Self::Ext2Mul => write!(f, "ext2mul"),
			Self::Ext2Neg => write!(f, "ext2neg"),
			Self::Ext2Sub => write!(f, "ext2sub"),
			Self::Err => write!(f, "err"),
			Self::Exec => write!(f, "exec"),
			Self::Exp => write!(f, "exp"),
			Self::ExpU => write!(f, "exp.u"),
			Self::Export => write!(f, "export"),
			Self::False => write!(f, "false"),
			Self::FriExt2Fold4 => write!(f, "fri_ext2fold4"),
			Self::Gt => write!(f, "gt"),
			Self::Gte => write!(f, "gte"),
			Self::Hash => write!(f, "hash"),
			Self::Hperm => write!(f, "hperm"),
			Self::Hmerge => write!(f, "hmerge"),
			Self::If => write!(f, "if"),
			Self::ILog2 => write!(f, "ilog2"),
			Self::Inv => write!(f, "inv"),
			Self::IsOdd => write!(f, "is_odd"),
			Self::Local => write!(f, "local"),
			Self::Locaddr => write!(f, "locaddr"),
			Self::LocLoad => write!(f, "loc_load"),
			Self::LocLoadw => write!(f, "loc_loadw"),
			Self::LocStore => write!(f, "loc_store"),
			Self::LocStorew => write!(f, "loc_storew"),
			Self::Lt => write!(f, "lt"),
			Self::Lte => write!(f, "lte"),
			Self::Mem => write!(f, "mem"),
			Self::MemLoad => write!(f, "mem_load"),
			Self::MemLoadw => write!(f, "mem_loadw"),
			Self::MemStore => write!(f, "mem_store"),
			Self::MemStorew => write!(f, "mem_storew"),
			Self::MemStream => write!(f, "mem_stream"),
			Self::Movdn => write!(f, "movdn"),
			Self::Movdnw => write!(f, "movdnw"),
			Self::Movup => write!(f, "movup"),
			Self::Movupw => write!(f, "movupw"),
			Self::MtreeGet => write!(f, "mtree_get"),
			Self::MtreeMerge => write!(f, "mtree_merge"),
			Self::MtreeSet => write!(f, "mtree_set"),
			Self::MtreeVerify => write!(f, "mtree_verify"),
			Self::Mul => write!(f, "mul"),
			Self::Neg => write!(f, "neg"),
			Self::Neq => write!(f, "neq"),
			Self::Not => write!(f, "not"),
			Self::Nop => write!(f, "nop"),
			Self::Or => write!(f, "or"),
			Self::Padw => write!(f, "padw"),
			Self::Pow2 => write!(f, "pow2"),
			Self::Proc => write!(f, "proc"),
			Self::Procref => write!(f, "procref"),
			Self::Push => write!(f, "push"),
			Self::RCombBase => write!(f, "rcomb_base"),
			Self::Repeat => write!(f, "repeat"),
			Self::RpoFalcon512 => write!(f, "rpo_falcon512"),
			Self::Sdepth => write!(f, "sdepth"),
			Self::Stack => write!(f, "stack"),
			Self::Sub => write!(f, "sub"),
			Self::Swap => write!(f, "swap"),
			Self::Swapw => write!(f, "swapw"),
			Self::Swapdw => write!(f, "swapdw"),
			Self::SysCall => write!(f, "syscall"),
			Self::Trace => write!(f, "trace"),
			Self::True => write!(f, "true"),
			Self::Use => write!(f, "use"),
			Self::U32And => write!(f, "u32and"),
			Self::U32Assert => write!(f, "u32assert"),
			Self::U32Assert2 => write!(f, "u32assert2"),
			Self::U32Assertw => write!(f, "u32assertw"),
			Self::U32Cast => write!(f, "u32cast"),
			Self::U32Div => write!(f, "u32div"),
			Self::U32Divmod => write!(f, "u32divmod"),
			Self::U32Gt => write!(f, "u32gt"),
			Self::U32Gte => write!(f, "u32gte"),
			Self::U32Lt => write!(f, "u32lt"),
			Self::U32Lte => write!(f, "u32lte"),
			Self::U32Max => write!(f, "u32max"),
			Self::U32Min => write!(f, "u32min"),
			Self::U32Mod => write!(f, "u32mod"),
			Self::U32Not => write!(f, "u32not"),
			Self::U32Or => write!(f, "u32or"),
			Self::U32OverflowingAdd => write!(f, "u32overflowing_add"),
			Self::U32OverflowingAdd3 => write!(f, "u32overflowing_add3"),
			Self::U32OverflowingMadd => write!(f, "u32overflowing_madd"),
			Self::U32OverflowingMul => write!(f, "u32overflowing_mul"),
			Self::U32OverflowingSub => write!(f, "u32overflowing_sub"),
			Self::U32Popcnt => write!(f, "u32popcnt"),
			Self::U32Clz => write!(f, "u32clz"),
			Self::U32Ctz => write!(f, "u32ctz"),
			Self::U32Clo => write!(f, "u32clo"),
			Self::U32Cto => write!(f, "u32cto"),
			Self::U32Rotl => write!(f, "u32rotl"),
			Self::U32Rotr => write!(f, "u32rotr"),
			Self::U32Shl => write!(f, "u32shl"),
			Self::U32Shr => write!(f, "u32shr"),
			Self::U32Split => write!(f, "u32split"),
			Self::U32Test => write!(f, "u32test"),
			Self::U32Testw => write!(f, "u32testw"),
			Self::U32WrappingAdd => write!(f, "u32wrapping_add"),
			Self::U32WrappingAdd3 => write!(f, "u32wrapping_add3"),
			Self::U32WrappingMadd => write!(f, "u32wrapping_madd"),
			Self::U32WrappingMul => write!(f, "u32wrapping_mul"),
			Self::U32WrappingSub => write!(f, "u32wrapping_sub"),
			Self::U32Xor => write!(f, "u32xor"),
			Self::While => write!(f, "while"),
			Self::Xor => write!(f, "xor"),
			Self::At => write!(f, "@"),
			Self::Bang => write!(f, "!"),
			Self::ColonColon => write!(f, "::"),
			Self::Dot => write!(f, "."),
			Self::Comma => write!(f, ","),
			Self::Equal => write!(f, "="),
			Self::Lparen => write!(f, "("),
			Self::Lbracket => write!(f, "["),
			Self::Minus => write!(f, "-"),
			Self::Plus => write!(f, "+"),
			Self::SlashSlash => write!(f, "//"),
			Self::Slash => write!(f, "/"),
			Self::Star => write!(f, "*"),
			Self::Rparen => write!(f, ")"),
			Self::Rbracket => write!(f, "]"),
			Self::Rstab => write!(f, "->"),
			Self::DocComment(DocumentationType::Module(_)) => f.write_str("module doc"),
			Self::DocComment(DocumentationType::Form(_)) => f.write_str("doc comment"),
			Self::HexValue(_) => f.write_str("hex-encoded value"),
			Self::BinValue(_) => f.write_str("bin-encoded value"),
			Self::Int(_) => f.write_str("integer"),
			Self::Ident(_) => f.write_str("identifier"),
			Self::ConstantIdent(_) => f.write_str("constant identifier"),
			Self::QuotedIdent(_) => f.write_str("quoted identifier"),
			Self::QuotedString(_) => f.write_str("quoted string"),
			Self::Comment => f.write_str("comment"),
			Self::Eof => write!(f, "end of file"),
		}
	}
}
