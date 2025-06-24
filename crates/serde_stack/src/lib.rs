#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod de;
mod param;
mod ser;

pub use self::{
	de::Deserializer,
	ser::{SerializeHelper, Serializer},
};

#[cfg(all(test, not(miri)))]
mod tests {
	use serde::{Deserialize, Serialize};
	use serde_json::{Result, Value};

	fn drop_carefully(value: Value) {
		let mut stack = vec![value];
		while let Some(value) = stack.pop() {
			if let Value::Array(array) = value {
				stack.extend(array);
			}
		}
	}

	#[test]
	fn deserialize_works() -> Result<()> {
		let mut json = String::new();
		for _ in 0..10000 {
			json = format!("[{json}]");
		}

		let mut deserializer = serde_json::Deserializer::from_str(&json);
		deserializer.disable_recursion_limit();
		let deserializer = super::Deserializer::new(&mut deserializer);
		let value = Value::deserialize(deserializer)?;

		drop_carefully(value);

		Ok(())
	}

	#[test]
	fn serialize_works() -> Result<()> {
		let mut value = Value::Null;
		for _ in 0..10000 {
			value = Value::Array(vec![value]);
		}

		let mut out = Vec::new();
		let mut serializer = serde_json::Serializer::new(&mut out);
		let serializer = super::Serializer::new(&mut serializer);
		let result = value.serialize(serializer);

		drop_carefully(value);

		result?;
		assert_eq!(out.len(), 10000 + 4 + 10000);

		Ok(())
	}
}
