use anyhow::Result;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::CompoundSerializer;
use crate::{Compound, List, compound};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
	foo: i32,
	bar: StructInner,
	baz: String,
	quux: Vec<f32>,
	blah: EnumInner,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct StructInner {
	a: bool,
	b: i64,
	c: Vec<Vec<i32>>,
	d: Vec<Self>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum EnumInner {
	A,
	B,
	C,
}

fn make_struct() -> Struct {
	Struct {
		foo: i32::MIN,
		bar: StructInner {
			a: true,
			b: 123_456_789,
			c: vec![vec![1, 2, 3], vec![4, 5, 6]],
			d: vec![],
		},
		baz: "🤨".to_owned(),
		quux: vec![std::f32::consts::PI, f32::MAX, f32::MIN],
		blah: EnumInner::B,
	}
}

fn make_compound() -> Compound {
	compound! {
		"foo" => i32::MIN,
		"bar" => compound! {
			"a" => true,
			"b" => 123_456_789_i64,
			"c" => List::IntArray(vec![vec![1, 2, 3], vec![4, 5, 6]]),
			"d" => List::End,
		},
		"baz" => "🤨",
		"quux" => List::Float(vec![
			std::f32::consts::PI,
			f32::MAX,
			f32::MIN,
		]),
		"blah" => "B"
	}
}

fn make_json() -> serde_json::Value {
	json!({
		"foo": i32::MIN,
		"bar": {
			"a": true,
			"b": 123_456_789_i64,
			"c": [[1, 2, 3], [4, 5, 6]],
			"d": []
		},
		"baz": "🤨",
		"quux": [
			std::f32::consts::PI,
			f32::MAX,
			f32::MIN,
		],
		"blah": "B"
	})
}

#[test]
fn struct_to_compound() -> Result<()> {
	let c = make_struct().serialize(CompoundSerializer)?;

	assert_eq!(c, make_compound());

	Ok(())
}

#[test]
fn compound_to_struct() -> Result<()> {
	let s = Struct::deserialize(make_compound())?;

	assert_eq!(s, make_struct());

	Ok(())
}

#[test]
fn compound_to_json() -> Result<()> {
	let mut j = serde_json::to_value(make_compound())?;

	let p = j.pointer_mut("/bar/a").unwrap();
	assert_eq!(*p, serde_json::Value::from(1));
	*p = true.into();

	assert_eq!(j, make_json());

	Ok(())
}
