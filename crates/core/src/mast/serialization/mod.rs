mod basic_block;
mod decorator;
mod info;
mod string_table;

const MAGIC: &[u8; 5] = b"MAST\0";

type NodeDataOffset = u32;
type DecoratorDataOffset = u32;
type StringDataOffset = usize;
type StringIndex = usize;
