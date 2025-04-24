use anyhow::Result;
use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

const NUMBER: usize = 137;

#[derive(Serialize, Deserialize)]
struct S {
	#[serde(with = "BigArray")]
	arr1: [u8; NUMBER * NUMBER + 17],
	#[serde(with = "BigArray")]
	arr2: [u8; NUMBER],
	#[serde(with = "BigArray")]
	arr3: [u8; 42],
}

#[test]
fn works() -> Result<()> {
	let s = S {
		arr1: [1; NUMBER * NUMBER + 17],
		arr2: [2; NUMBER],
		arr3: [3; 42],
	};

    let v = serde_value::to_value(&s)?;
    let s_back = v.deserialize_into::<S>()?;

    assert_eq!(&s.arr1[..], &s_back.arr1[..]);
    assert_eq!(&s.arr2[..], &s_back.arr2[..]);
    assert_eq!(&s.arr3[..], &s_back.arr3[..]);

	Ok(())
}
