use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_array::BigArray;

mod module {
	pub const NUMBER: usize = 127;
}

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct S {
	#[serde(with = "BigArray")]
	arr: [u8; module::NUMBER],
}

#[test]
fn works() -> Result<()> {
	let s = S {
		arr: [1; module::NUMBER],
	};

	let v = serde_value::to_value(&s)?;
	let s_back = v.deserialize_into::<S>()?;
	assert_eq!(&s.arr[..], &s_back.arr[..]);

	Ok(())
}
