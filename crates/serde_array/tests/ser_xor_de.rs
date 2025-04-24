use anyhow::Result;
use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

#[derive(Debug, Clone, Copy, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
struct SerOnly(u8);

impl PartialEq<DeOnly> for SerOnly {
	fn eq(&self, other: &DeOnly) -> bool {
		self.0 == other.0
	}
}

#[derive(Serialize)]
#[repr(transparent)]
#[serde(transparent)]
struct S {
	#[serde(with = "BigArray")]
	arr: [SerOnly; 64],
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct DeOnly(u8);

#[derive(Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct D {
	#[serde(with = "BigArray")]
	arr: [DeOnly; 64],
}

#[test]
fn works() -> Result<()> {
    let s = S {
        arr: [SerOnly(1); 64]
    };

    let v = serde_value::to_value(&s)?;
    let s_back = v.deserialize_into::<D>()?;
    assert_eq!(&s.arr[..], &s_back.arr[..]);

    Ok(())
}
