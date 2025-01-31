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
