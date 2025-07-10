use alloc::{borrow::ToOwned, vec};
use core::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};
use serde_bytes::ByteBuf;

use super::*;
use crate::{Result, Type, tests::init_tracing};

#[tracing::instrument(skip(expected))]
fn round_trip<T>(value: &T, expected: &Value<'_>) -> Result<()>
where
	T: Debug + DeserializeOwned + PartialEq + Serialize,
{
	let ir = to_value(&value)?;
	assert_eq!(ir, *expected);

	let bytes = crate::to_vec(&value)?;
	tracing::info!("bytes repr: {bytes:?}");

	let ir_value: OwnedValue = crate::from_slice(bytes.as_slice())?;
	let ir_value = ir_value.into_inner();

	assert_eq!(ir_value, *expected);

	let deserialized: T = from_value(ir_value)?;

	assert_eq!(deserialized, *value);

	Ok(())
}

#[tracing::instrument(skip(expected))]
fn round_trip_with_indices<T>(value: &T, expected: &Value<'_>) -> Result<()>
where
	T: Debug + DeserializeOwned + PartialEq + Serialize,
{
	let config = Config::new(true, true, 0);
	let ir = to_value_with_config(&value, config)?;
	assert_eq!(ir, *expected);

	let bytes = crate::to_vec_with_config(&value, config)?;
	tracing::info!("bytes repr: {bytes:?}");

	let ir_value: OwnedValue = crate::from_slice_with_config(bytes.as_slice(), config)?;
	let ir_value = ir_value.into_inner();

	assert_eq!(ir_value, *expected);

	let deserialized: T = from_value_with_config(ir_value, config)?;

	assert_eq!(deserialized, *value);

	Ok(())
}

#[tracing::instrument(skip(expected))]
fn deser<'de, T>(bytes: &'de [u8], expected: &Value<'_>) -> Result<()>
where
	T: Debug + Deserialize<'de> + Serialize,
{
	let value: Value<'de> = crate::from_slice(bytes)?;
	assert_eq!(value, *expected);
	let deserialized: T = from_value(value)?;
	tracing::info!("deserialized value: {deserialized:?}");

	let value = to_value(&deserialized)?;
	assert_eq!(value, *expected);
	let serialized = crate::to_vec(&value)?;
	assert_eq!(serialized, bytes);

	Ok(())
}

#[tracing::instrument(skip(expected))]
fn deser_with_indices<'de, T>(bytes: &'de [u8], expected: &Value<'_>) -> Result<()>
where
	T: Debug + Deserialize<'de> + Serialize,
{
	let config = Config::new(true, true, 0);

	let value: Value<'de> = crate::from_slice_with_config(bytes, config)?;
	assert_eq!(value, *expected);
	let deserialized: T = from_value_with_config(value, config)?;
	tracing::info!("deserialized value: {deserialized:?}");

	let value = to_value_with_config(&deserialized, config)?;
	assert_eq!(value, *expected);
	let serialized = crate::to_vec_with_config(&value, config)?;
	assert_eq!(serialized, bytes);

	Ok(())
}

#[test]
fn unit() -> Result<()> {
	init_tracing();

	round_trip(&(), &Value::Null)?;
	round_trip_with_indices(&(), &Value::Null)?;

	deser::<()>(&[Type::Null.into()], &Value::Null)?;
	deser_with_indices::<()>(&[Type::Null.into()], &Value::Null)
}
