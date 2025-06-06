use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_array::Array;

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct S {
	arr: Array<Array<u8, 234>, 65>,
}

#[test]
fn works() -> Result<()> {
	let s = S {
		arr: Array([Array([1; 234]); 65]),
	};

	let v = serde_value::to_value(&s)?;
	let s_back = v.deserialize_into::<S>()?;
	assert_eq!(&s.arr[..], &s_back.arr[..]);

	Ok(())
}
