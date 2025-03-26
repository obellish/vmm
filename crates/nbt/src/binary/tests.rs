use anyhow::Result;

use crate::{Compound, List, Tag, Value, compound, from_binary, to_binary};

const ROOT_NAME: &str = "The root name!";

fn example_compound() -> Compound {
	fn inner() -> Compound {
		compound! {
			"int" => i32::MIN,
			"long" => i64::MAX,
			"float" => 1e10_f32,
			"double" => f64::INFINITY,
		}
	}

	compound! {
		"byte" => 123_i8,
		"list_of_int" => List::Int(vec![3, -7, 5]),
		"list_of_string" => List::String(vec![
			"foo".to_owned(),
			"bar".to_owned(),
			"baz".to_owned()
		]),
		"list_of_end" => List::End,
		"string" => "aé日",
		"compound" => inner(),
		"list_of_compound" => List::Compound(vec![
			inner(),
			inner(),
			inner(),
		]),
		"int_array" => vec![5, -9, i32::MIN, 0, i32::MAX],
		"byte_array" => vec![0_i8, 2, 3],
		"long_array" => vec![123_i64, 456, 789],
	}
}

fn check(min_val: Value, expected_size: usize) -> Result<()> {
	const COMPOUND_OVERHEAD: usize = 1 + 2 + 1 + 2 + 1;

	let dbg = format!("{min_val:?}");
	let mut buf = Vec::new();

	to_binary(&compound!("" => min_val), &mut buf, "")?;

	assert_eq!(
		expected_size,
		buf.len() - COMPOUND_OVERHEAD,
		"size mismatch for {dbg}"
	);

	Ok(())
}

#[test]
fn round_trip() -> Result<()> {
	let mut buf = Vec::new();

	let compound = example_compound();

	to_binary(&compound, &mut buf, ROOT_NAME)?;

	println!("{buf:?}");

	let (decoded, root_name) = from_binary(&mut buf.as_slice())?;

	assert_eq!(root_name, ROOT_NAME);
	assert_eq!(compound, decoded);

	Ok(())
}

#[test]
fn check_min_sizes() -> Result<()> {
	check(Value::Byte(0), 1)?;
	check(Value::Short(0), 2)?;
	check(Value::Int(0), 4)?;
	check(Value::Long(0), 8)?;
	check(Value::Float(0.0), 4)?;
	check(Value::Double(0.0), 8)?;
	check(Value::ByteArray(Vec::new()), 4)?;
	check(Value::String(String::new()), 2)?;
	check(Value::List(Vec::<i32>::new().into()), 5)?;
	check(Value::Compound(compound!()), 1)?;
	check(Value::IntArray(Vec::new()), 4)?;
	check(Value::LongArray(Vec::new()), 4)?;

	Ok(())
}

#[test]
fn deeply_nested_compound_decode() {
	let mut buf = vec![Tag::Compound as u8, 0, 0];
	let n = 10_000;

	for _ in 0..n {
		buf.extend([Tag::Compound as u8, 0, 0]);
	}

	buf.extend((0..n).map(|_| Tag::End as u8));

	buf.push(Tag::End as u8);

	_ = from_binary::<String>(&mut buf.as_slice());
}

#[test]
fn deeply_nested_list_decode() {
	let mut buf = vec![Tag::Compound as u8, 0, 0, Tag::List as u8, 0, 0];
	let n = 10_000;

	for _ in 0..n - 1 {
		buf.extend([Tag::List as u8, 0, 0, 0, 1]);
	}

	buf.extend([Tag::Byte as u8, 0, 0, 0, 0]);

	buf.push(Tag::End as u8);

	_ = from_binary::<String>(&mut buf.as_slice());
}
