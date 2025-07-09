mod basic_types;
mod serde_attributes;

use core::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::{Config, Result, from_slice, to_slice};
use crate::{from_slice_with_config, to_slice_with_config};

#[cfg(feature = "std")]
pub fn init_tracing() {
	static INITIALIZED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

	INITIALIZED.get_or_init(|| {
		tracing_subscriber::fmt()
			.with_test_writer()
			.with_max_level(tracing::Level::TRACE)
			.pretty()
			.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
			.init();
	});
}

#[cfg(not(feature = "std"))]
pub fn init_tracing() {}

#[tracing::instrument(skip(buffer))]
fn round_trip<'de, T>(value: &T, buffer: &'de mut [u8]) -> Result<()>
where
	T: Debug + Deserialize<'de> + PartialEq + Serialize,
{
	let bytes = to_slice(&value, buffer)?;
	tracing::info!("bytes repr: {bytes:?}");
	let deserialized = from_slice(bytes)?;

	assert_eq!(*value, deserialized);

	Ok(())
}

#[tracing::instrument(skip(buffer))]
fn round_trip_with_indices<'de, T>(value: &T, buffer: &'de mut [u8]) -> Result<()>
where
	T: Debug + Deserialize<'de> + PartialEq + Serialize,
{
	let config = Config::new(true, true, 0);

	let bytes = to_slice_with_config(value, buffer, config)?;
	tracing::info!("bytes repr: {bytes:?}");
	let deserialized = from_slice_with_config(bytes, config)?;

	assert_eq!(*value, deserialized);

	Ok(())
}

#[tracing::instrument]
fn deser<'de, T>(bytes: &'de [u8]) -> Result<()>
where
	T: Debug + Deserialize<'de> + Serialize,
{
	let deserialized = from_slice::<T>(bytes)?;
	tracing::info!("deserialized value: {deserialized:?}");

	let mut buffer = [0; 1024];
	let serialized = to_slice(&deserialized, &mut buffer)?;

	assert_eq!(bytes, serialized);

	Ok(())
}

#[tracing::instrument]
fn deser_with_indices<'de, T>(bytes: &'de [u8]) -> Result<()>
where
	T: Debug + Deserialize<'de> + Serialize,
{
	let config = Config::new(true, true, 0);
	let deserialized = from_slice_with_config::<T>(bytes, config)?;
	tracing::info!("deserialized value: {deserialized:?}");

	let mut buffer = [0; 1024];
	let serialized = to_slice_with_config(&deserialized, &mut buffer, config)?;
	assert_eq!(bytes, serialized);

	Ok(())
}
