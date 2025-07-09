use serde_bytes::Bytes;
use serde_derive::{Deserialize, Serialize};

use super::{deser, deser_with_indices, init_tracing, round_trip, round_trip_with_indices};
use crate::{Result, Type, to_slice};

#[test]
fn unit() -> Result<()> {
	init_tracing();

	round_trip(&(), &mut [0; 1024])?;
	round_trip_with_indices(&(), &mut [0; 1024])?;
	deser::<()>(&[Type::Null.into()])?;
	deser_with_indices::<()>(&[Type::Null.into()])
}

#[test]
fn bool() -> Result<()> {
	init_tracing();

	round_trip(&true, &mut [0; 1024])?;
	round_trip_with_indices(&true, &mut [0; 1024])?;
	deser::<bool>(&[Type::True.into()])?;
	deser_with_indices::<bool>(&[Type::True.into()])?;

	round_trip(&false, &mut [0; 1024])?;
	round_trip_with_indices(&false, &mut [0; 1024])?;
	deser::<bool>(&[Type::False.into()])?;
	deser_with_indices::<bool>(&[Type::False.into()])
}

#[test]
fn unsigned_int() -> Result<()> {
	init_tracing();

	round_trip(&125u8, &mut [0; 1024])?;
	round_trip_with_indices(&125u8, &mut [0; 1024])?;

	round_trip(&125usize, &mut [0; 1024])?;
	round_trip_with_indices(&125usize, &mut [0; 1024])?;

	round_trip(&125u128, &mut [0; 1024])?;
	round_trip_with_indices(&125u128, &mut [0; 1024])?;

	round_trip(&0x0123_4567_89AB_CDEFu64, &mut [0; 1024])?;
	round_trip_with_indices(&0x0123_4567_89AB_CDEFu64, &mut [0; 1024])?;

	deser::<usize>(&[Type::UnsignedInt.into(), 0x00])?;
	deser_with_indices::<usize>(&[Type::UnsignedInt.into(), 0x00])?;

	deser::<usize>(&[Type::UnsignedInt.into(), 0xFF, 0x01])?;
	deser_with_indices::<usize>(&[Type::UnsignedInt.into(), 0xFF, 0x01])
}

#[test]
fn signed_int() -> Result<()> {
	init_tracing();

	round_trip(&125i8, &mut [0; 1024])?;
	round_trip_with_indices(&125i8, &mut [0; 1024])?;

	round_trip(&-125isize, &mut [0; 1024])?;
	round_trip_with_indices(&-125isize, &mut [0; 1024])?;

	round_trip(&125isize, &mut [0; 1024])?;
	round_trip_with_indices(&125isize, &mut [0; 1024])?;

	round_trip(&0x0123_4567_89AB_CDEFi64, &mut [0; 1024])?;
	round_trip_with_indices(&0x0123_4567_89AB_CDEFi64, &mut [0; 1024])?;

	round_trip(&-0x0123_4567_89AB_CDEFi64, &mut [0; 1024])?;
	round_trip_with_indices(&-0x0123_4567_89AB_CDEFi64, &mut [0; 1024])?;

	deser::<isize>(&[Type::SignedInt.into(), 0x00])?;
	deser_with_indices::<isize>(&[Type::SignedInt.into(), 0x00])?;

	deser::<isize>(&[Type::SignedInt.into(), 0x01])?;
	deser_with_indices::<isize>(&[Type::SignedInt.into(), 0x01])?;

	deser::<isize>(&[Type::SignedInt.into(), 0xFE, 0x01])?;
	deser_with_indices::<isize>(&[Type::SignedInt.into(), 0xFE, 0x01])?;

	deser::<isize>(&[Type::SignedInt.into(), 0xFF, 0x01])?;
	deser_with_indices::<isize>(&[Type::SignedInt.into(), 0xFF, 0x01])
}

#[test]
fn floats() -> Result<()> {
	init_tracing();

	round_trip(&3.5f32, &mut [0; 1024])?;
	round_trip_with_indices(&3.5f32, &mut [0; 1024])?;

	round_trip(&3.5f64, &mut [0; 1024])?;
	round_trip_with_indices(&3.5f64, &mut [0; 1024])?;

	round_trip(&-3.5f64, &mut [0; 1024])?;
	round_trip_with_indices(&-3.5f64, &mut [0; 1024])?;

	deser::<f32>(&[Type::Float32.into(), 0x12, 0x34, 0x56, 0x78])?;
	deser_with_indices::<f32>(&[Type::Float32.into(), 0x12, 0x34, 0x56, 0x78])?;

	deser::<f64>(&[Type::Float64.into(), 1, 2, 3, 4, 5, 6, 7, 8])?;
	deser_with_indices::<f64>(&[Type::Float64.into(), 1, 2, 3, 4, 5, 6, 7, 8])
}

#[test]
fn bytes() -> Result<()> {
	init_tracing();

	round_trip(&Bytes::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), &mut [0; 1024])?;
	round_trip_with_indices(&Bytes::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), &mut [0; 1024])?;

	deser::<&Bytes>(&[Type::Bytes.into(), 5, 1, 2, 3, 4, 5])?;
	deser_with_indices::<&Bytes>(&[Type::Bytes.into(), 5, 1, 2, 3, 4, 5])
}

#[test]
fn string() -> Result<()> {
	init_tracing();

	round_trip(&"I was serialized and deserialized!", &mut [0; 1024])?;
	round_trip_with_indices(&"I was serialized and deserialized!", &mut [0; 1024])?;

	deser::<&str>(&[Type::String.into(), 5, b'H', b'e', b'l', b'l', b'o'])?;
	deser_with_indices::<&str>(&[Type::String.into(), 5, b'H', b'e', b'l', b'l', b'o'])
}

#[test]
fn char() -> Result<()> {
	init_tracing();

	round_trip(&'😻', &mut [0; 1024])?;
	round_trip_with_indices(&'😻', &mut [0; 1024])?;

	deser::<char>(&[Type::String.into(), 1, b'x'])?;
	deser_with_indices::<char>(&[Type::String.into(), 1, b'x'])
}

#[test]
fn sequence() -> Result<()> {
	init_tracing();

	round_trip(&[0; 0], &mut [0; 1024])?;
	round_trip_with_indices(&[0; 0], &mut [0; 1024])?;

	round_trip(&[true, false, true, false], &mut [0; 1024])?;
	round_trip_with_indices(&[true, false, true, false], &mut [0; 1024])?;

	deser::<[bool; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()])?;
	deser_with_indices::<[bool; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()])?;

	deser::<[Option<bool>; 3]>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::False.into(),
		Type::True.into(),
		Type::SeqEnd.into(),
	])?;
	deser_with_indices::<[Option<bool>; 3]>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::False.into(),
		Type::True.into(),
		Type::SeqEnd.into(),
	])
}

#[test]
fn tuple() -> Result<()> {
	init_tracing();

	round_trip(&(1,), &mut [0; 1024])?;
	round_trip_with_indices(&(1,), &mut [0; 1024])?;

	round_trip(&(1, 'h', 'i', 8), &mut [0; 1024])?;
	round_trip_with_indices(&(1, 'h', 'i', 8), &mut [0; 1024])?;

	deser::<(&str,)>(&[
		Type::SeqStart.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	])?;
	deser_with_indices::<(&str,)>(&[
		Type::SeqStart.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	])?;

	deser::<((), bool, Option<bool>, &str)>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::False.into(),
		Type::True.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	])?;
	deser_with_indices::<((), bool, Option<bool>, &str)>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::False.into(),
		Type::True.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	])
}

#[cfg(feature = "alloc")]
#[test]
fn map() -> Result<()> {
	use alloc::collections::BTreeMap;

	init_tracing();

	let mut map = BTreeMap::new();
	map.insert("null", 0);
	map.insert("one", 1);
	map.insert("two", 2);
	map.insert("three", 3);
	map.insert("four", 4);

	round_trip(&map, &mut [0; 1024])?;
	round_trip_with_indices(&map, &mut [0; 1024])?;

	deser::<BTreeMap<&str, &str>>(&[Type::MapStart.into(), Type::MapEnd.into()])?;
	deser_with_indices::<BTreeMap<&str, &str>>(&[Type::MapStart.into(), Type::MapEnd.into()])?;

	deser::<BTreeMap<bool, &str>>(&[
		Type::MapStart.into(),
		Type::False.into(),
		Type::String.into(),
		0,
		Type::MapEnd.into(),
	])?;
	deser_with_indices::<BTreeMap<bool, &str>>(&[
		Type::MapStart.into(),
		Type::False.into(),
		Type::String.into(),
		0,
		Type::MapEnd.into(),
	])
}

#[test]
fn option() -> Result<()> {
	init_tracing();

	round_trip(&None::<bool>, &mut [0; 1024])?;
	round_trip_with_indices(&None::<bool>, &mut [0; 1024])?;

	round_trip(&Some(true), &mut [0; 1024])?;
	round_trip_with_indices(&Some(true), &mut [0; 1024])?;

	round_trip(&None::<char>, &mut [0; 1024])?;
	round_trip_with_indices(&None::<char>, &mut [0; 1024])?;

	round_trip(&Some('a'), &mut [0; 1024])?;
	round_trip_with_indices(&Some('a'), &mut [0; 1024])?;

	round_trip(&None::<i32>, &mut [0; 1024])?;
	round_trip_with_indices(&None::<i32>, &mut [0; 1024])?;

	round_trip(&Some(5), &mut [0; 1024])?;
	round_trip_with_indices(&Some(5), &mut [0; 1024])?;

	deser::<Option<i32>>(&[Type::Null.into()])?;
	deser_with_indices::<Option<i32>>(&[Type::Null.into()])?;

	deser::<Option<i32>>(&[Type::SignedInt.into(), 5])?;
	deser_with_indices::<Option<i32>>(&[Type::SignedInt.into(), 5])
}

#[test]
fn format_args() -> Result<()> {
	init_tracing();

	let mut buffer = [0; 1024];
	let bytes = to_slice(&format_args!("XYZ {}", 5), &mut buffer)?;

	assert_eq!(
		bytes,
		&[Type::String.into(), 5, b'X', b'Y', b'Z', b' ', b'5']
	);

	Ok(())
}

#[test]
fn empty_struct() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Empty {}

	init_tracing();

	round_trip(&Empty {}, &mut [0; 1024])?;
	round_trip_with_indices(&Empty {}, &mut [0; 1024])?;

	deser::<Empty>(&[Type::MapStart.into(), Type::MapEnd.into()])?;
	deser_with_indices::<Empty>(&[Type::MapStart.into(), Type::MapEnd.into()])
}

#[test]
fn r#struct() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Struct {
		a: bool,
		b: bool,
	}

	init_tracing();

	round_trip(&Struct { a: false, b: true }, &mut [0; 1024])?;
	round_trip_with_indices(&Struct { a: false, b: true }, &mut [0; 1024])?;

	deser::<Struct>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::False.into(),
		Type::String.into(),
		1,
		b'b',
		Type::True.into(),
		Type::MapEnd.into(),
	])?;

	deser_with_indices::<Struct>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::False.into(),
		Type::UnsignedInt.into(),
		1,
		Type::True.into(),
		Type::MapEnd.into(),
	])
}

#[test]
fn newtype_struct() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Newtype(bool);

	init_tracing();

	round_trip(&Newtype(false), &mut [0; 1024])?;
	round_trip_with_indices(&Newtype(false), &mut [0; 1024])?;

	deser::<Newtype>(&[Type::True.into()])?;
	deser_with_indices::<Newtype>(&[Type::True.into()])
}

#[test]
fn tuple_struct() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Tuple(bool, bool);

	init_tracing();

	round_trip(&Tuple(false, true), &mut [0; 1024])?;
	round_trip_with_indices(&Tuple(false, true), &mut [0; 1024])?;

	deser::<Tuple>(&[
		Type::SeqStart.into(),
		Type::False.into(),
		Type::True.into(),
		Type::SeqEnd.into(),
	])?;
	deser_with_indices::<Tuple>(&[
		Type::SeqStart.into(),
		Type::False.into(),
		Type::True.into(),
		Type::SeqEnd.into(),
	])
}

#[test]
fn enums() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	enum Enum {
		Unit,
		Newtype(bool),
		Tuple(bool, bool),
		Struct { a: bool, b: bool },
	}

	init_tracing();

	round_trip(&Enum::Unit, &mut [0; 1024])?;
	round_trip_with_indices(&Enum::Unit, &mut [0; 1024])?;

	round_trip(&Enum::Newtype(false), &mut [0; 1024])?;
	round_trip_with_indices(&Enum::Newtype(false), &mut [0; 1024])?;

	round_trip(&Enum::Tuple(false, true), &mut [0; 1024])?;
	round_trip_with_indices(&Enum::Tuple(false, true), &mut [0; 1024])?;

	round_trip(&Enum::Struct { a: false, b: true }, &mut [0; 1024])?;
	round_trip_with_indices(&Enum::Struct { a: false, b: true }, &mut [0; 1024])?;

	round_trip(
		&[
			Enum::Unit,
			Enum::Newtype(true),
			Enum::Tuple(true, false),
			Enum::Struct { a: true, b: false },
		],
		&mut [0; 1024],
	)?;

	round_trip_with_indices(
		&[
			Enum::Unit,
			Enum::Newtype(true),
			Enum::Tuple(true, false),
			Enum::Struct { a: true, b: false },
		],
		&mut [0; 1024],
	)?;

	deser::<Enum>(&[Type::String.into(), 4, b'U', b'n', b'i', b't'])?;

	deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		7,
		b'N',
		b'e',
		b'w',
		b't',
		b'y',
		b'p',
		b'e',
		Type::True.into(),
		Type::MapEnd.into(),
	])?;

	deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		5,
		b'T',
		b'u',
		b'p',
		b'l',
		b'e',
		Type::SeqStart.into(),
		Type::True.into(),
		Type::False.into(),
		Type::SeqEnd.into(),
		Type::MapEnd.into(),
	])?;
	deser_with_indices::<Enum>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		2,
		Type::SeqStart.into(),
		Type::True.into(),
		Type::False.into(),
		Type::SeqEnd.into(),
		Type::MapEnd.into(),
	])?;

	deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		6,
		b'S',
		b't',
		b'r',
		b'u',
		b'c',
		b't',
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::False.into(),
		Type::String.into(),
		1,
		b'b',
		Type::True.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	])?;
	deser_with_indices::<Enum>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		3,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::False.into(),
		Type::UnsignedInt.into(),
		1,
		Type::True.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	])
}

#[test]
fn enums_with_discriminants() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	enum Enum {
		A = 5,
		B = 10,
	}

	init_tracing();

	round_trip(&Enum::A, &mut [0; 1024])?;
	round_trip_with_indices(&Enum::A, &mut [0; 1024])?;

	round_trip(&Enum::B, &mut [0; 1024])?;
	round_trip_with_indices(&Enum::B, &mut [0; 1024])?;

	deser::<Enum>(&[Type::String.into(), 1, b'A'])?;
	deser_with_indices::<Enum>(&[Type::UnsignedInt.into(), 0])?;

	deser::<Enum>(&[Type::String.into(), 1, b'B'])?;
	deser_with_indices::<Enum>(&[Type::UnsignedInt.into(), 1])
}

#[test]
fn all_numbers() -> Result<()> {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Numbers {
		u_8: u8,
		u_16: u16,
		u_32: u32,
		u_64: u64,
		u_128: u128,
		u_size: usize,
		i_8: i8,
		i_16: i16,
		i_32: i32,
		i_64: i64,
		i_128: i128,
		i_size: isize,
		f_32: f32,
		f_64: f64,
	}

	init_tracing();

	round_trip(
		&Numbers {
			u_8: 0,
			u_16: 0,
			u_32: 0,
			u_64: 0,
			u_128: 0,
			u_size: 0,
			i_8: 0,
			i_16: 0,
			i_32: 0,
			i_64: 0,
			i_128: 0,
			i_size: 0,
			f_32: 0.0,
			f_64: 0.0,
		},
		&mut [0; 1024],
	)?;
	round_trip_with_indices(
		&Numbers {
			u_8: 0,
			u_16: 0,
			u_32: 0,
			u_64: 0,
			u_128: 0,
			u_size: 0,
			i_8: 0,
			i_16: 0,
			i_32: 0,
			i_64: 0,
			i_128: 0,
			i_size: 0,
			f_32: 0.0,
			f_64: 0.0,
		},
		&mut [0; 1024],
	)?;

	round_trip(
		&Numbers {
			u_8: u8::MIN,
			u_16: u16::MIN,
			u_32: u32::MIN,
			u_64: u64::MIN,
			u_128: u128::MIN,
			u_size: usize::MIN,
			i_8: i8::MIN,
			i_16: i16::MIN,
			i_32: i32::MIN,
			i_64: i64::MIN,
			i_128: i128::MIN,
			i_size: isize::MIN,
			f_32: f32::MIN,
			f_64: f64::MIN,
		},
		&mut [0; 1024],
	)?;
	round_trip_with_indices(
		&Numbers {
			u_8: u8::MIN,
			u_16: u16::MIN,
			u_32: u32::MIN,
			u_64: u64::MIN,
			u_128: u128::MIN,
			u_size: usize::MIN,
			i_8: i8::MIN,
			i_16: i16::MIN,
			i_32: i32::MIN,
			i_64: i64::MIN,
			i_128: i128::MIN,
			i_size: isize::MIN,
			f_32: f32::MIN,
			f_64: f64::MIN,
		},
		&mut [0; 1024],
	)?;

	round_trip(
		&Numbers {
			u_8: u8::MAX,
			u_16: u16::MAX,
			u_32: u32::MAX,
			u_64: u64::MAX,
			u_128: u128::MAX,
			u_size: usize::MAX,
			i_8: i8::MAX,
			i_16: i16::MAX,
			i_32: i32::MAX,
			i_64: i64::MAX,
			i_128: i128::MAX,
			i_size: isize::MAX,
			f_32: f32::MAX,
			f_64: f64::MAX,
		},
		&mut [0; 1024],
	)?;
	round_trip_with_indices(
		&Numbers {
			u_8: u8::MAX,
			u_16: u16::MAX,
			u_32: u32::MAX,
			u_64: u64::MAX,
			u_128: u128::MAX,
			u_size: usize::MAX,
			i_8: i8::MAX,
			i_16: i16::MAX,
			i_32: i32::MAX,
			i_64: i64::MAX,
			i_128: i128::MAX,
			i_size: isize::MAX,
			f_32: f32::MAX,
			f_64: f64::MAX,
		},
		&mut [0; 1024],
	)?;

	deser::<Numbers>(&[
		Type::MapStart.into(),
		Type::String.into(),
		3,
		b'u',
		b'_',
		b'8',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'u',
		b'_',
		b'1',
		b'6',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'u',
		b'_',
		b'3',
		b'2',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'u',
		b'_',
		b'6',
		b'4',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		5,
		b'u',
		b'_',
		b'1',
		b'2',
		b'8',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		6,
		b'u',
		b'_',
		b's',
		b'i',
		b'z',
		b'e',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		3,
		b'i',
		b'_',
		b'8',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'i',
		b'_',
		b'1',
		b'6',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'i',
		b'_',
		b'3',
		b'2',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'i',
		b'_',
		b'6',
		b'4',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		5,
		b'i',
		b'_',
		b'1',
		b'2',
		b'8',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		6,
		b'i',
		b'_',
		b's',
		b'i',
		b'z',
		b'e',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'f',
		b'_',
		b'3',
		b'2',
		Type::Float32.into(),
		1,
		2,
		3,
		4,
		Type::String.into(),
		4,
		b'f',
		b'_',
		b'6',
		b'4',
		Type::Float64.into(),
		1,
		2,
		3,
		4,
		5,
		6,
		7,
		8,
		Type::MapEnd.into(),
	])?;
	deser_with_indices::<Numbers>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		1,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		2,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		3,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		4,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		5,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		6,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		7,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		8,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		9,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		10,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		11,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		12,
		Type::Float32.into(),
		1,
		2,
		3,
		4,
		Type::UnsignedInt.into(),
		13,
		Type::Float64.into(),
		1,
		2,
		3,
		4,
		5,
		6,
		7,
		8,
		Type::MapEnd.into(),
	])
}

#[test]
fn nested_sequence() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Inner {
		a: bool,
		b: bool,
	}

	init_tracing();

	round_trip(
		&[Inner { a: false, b: true }, Inner { a: true, b: false }],
		&mut [0; 1024],
	)?;
	round_trip_with_indices(
		&[Inner { a: false, b: true }, Inner { a: true, b: false }],
		&mut [0; 1024],
	)?;

	round_trip(&[Inner { a: false, b: true }], &mut [0; 1024])?;
	round_trip_with_indices(&[Inner { a: false, b: true }], &mut [0; 1024])?;

	deser::<[Inner; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()])?;
	deser_with_indices::<[Inner; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()])?;

	deser::<[Inner; 1]>(&[
		Type::SeqStart.into(),
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::False.into(),
		Type::String.into(),
		1,
		b'b',
		Type::True.into(),
		Type::MapEnd.into(),
		Type::SeqEnd.into(),
	])?;
	deser_with_indices::<[Inner; 1]>(&[
		Type::SeqStart.into(),
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::False.into(),
		Type::UnsignedInt.into(),
		1,
		Type::True.into(),
		Type::MapEnd.into(),
		Type::SeqEnd.into(),
	])
}

#[test]
fn nested_struct() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Inner {
		a: bool,
		b: bool,
	}

	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	struct Outer {
		a: bool,
		b: Inner,
		c: Inner,
	}

	init_tracing();

	round_trip(
		&Outer {
			a: false,
			b: Inner { a: true, b: true },
			c: Inner { a: false, b: false },
		},
		&mut [0; 1024],
	)?;
	round_trip_with_indices(
		&Outer {
			a: false,
			b: Inner { a: true, b: true },
			c: Inner { a: false, b: false },
		},
		&mut [0; 1024],
	)?;

	deser::<Outer>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::True.into(),
		Type::String.into(),
		1,
		b'b',
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::False.into(),
		Type::String.into(),
		1,
		b'b',
		Type::True.into(),
		Type::MapEnd.into(),
		Type::String.into(),
		1,
		b'c',
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::False.into(),
		Type::String.into(),
		1,
		b'b',
		Type::True.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	])?;
	deser_with_indices::<Outer>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::True.into(),
		Type::UnsignedInt.into(),
		1,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::False.into(),
		Type::UnsignedInt.into(),
		1,
		Type::True.into(),
		Type::MapEnd.into(),
		Type::UnsignedInt.into(),
		2,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::False.into(),
		Type::UnsignedInt.into(),
		1,
		Type::True.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	])
}
