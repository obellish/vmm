use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_array::BigArray;

const NUMBER: usize = 127;

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct S {
	#[serde(with = "BigArray")]
	arr: [u8; NUMBER],
}

#[test]
fn works() -> Result<()> {
	let s = S { arr: [1; NUMBER] };
	let v = serde_value::to_value(&s)?;
	let s_back = v.deserialize_into::<S>()?;

	assert_eq!(&s.arr[..], &s_back.arr[..]);

	Ok(())
}
