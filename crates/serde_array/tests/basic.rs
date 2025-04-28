use anyhow::Result;
use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

#[derive(Serialize, Deserialize)]
struct S {
	#[serde(with = "BigArray")]
	arr: [u8; 64],
}

#[test]
fn works() -> Result<()> {
	let s = S { arr: [1; 64] };
	let v = serde_value::to_value(&s)?;
	let s_back = v.deserialize_into::<S>()?;

	assert_eq!(&s.arr[..], &s_back.arr[..]);

	Ok(())
}
