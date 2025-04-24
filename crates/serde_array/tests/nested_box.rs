use anyhow::Result;
use serde::{Deserialize, Serialize};
use vmm_serde_array::Array;

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct S {
	arr: Box<Array<u8, 1234>>,
}

#[test]
fn works() -> Result<()> {
	let s = S {
		arr: Box::new(Array([1; 1234])),
	};

	let v = serde_value::to_value(&s)?;
	let s_back = v.deserialize_into::<S>()?;
	assert_eq!(&s.arr[..], &s_back.arr[..]);
	Ok(())
}
