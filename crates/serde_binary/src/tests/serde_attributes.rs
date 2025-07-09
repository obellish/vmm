use serde_derive::{Deserialize, Serialize};

use super::{deser, init_tracing, round_trip, round_trip_with_indices};
use crate::{Result, Type};

#[test]
fn rename_struct() -> Result<()> {
	#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
	#[serde(rename = "V1", rename_all = "PascalCase")]
	struct Struct {
		first_field: bool,
		#[serde(rename = "Extension")]
		second_field: bool,
	}

	init_tracing();

	round_trip(
		&Struct {
			first_field: true,
			second_field: false,
		},
		&mut [0; 1024],
	)?;
	round_trip_with_indices(
		&Struct {
			first_field: true,
			second_field: false,
		},
		&mut [0; 1024],
	)?;

	deser::<Struct>(&[
		Type::MapStart.into(),
		Type::String.into(),
		10,
		b'F',
		b'i',
		b'r',
		b's',
		b't',
		b'F',
		b'i',
		b'e',
		b'l',
		b'd',
		Type::True.into(),
		Type::String.into(),
		9,
		b'E',
		b'x',
		b't',
		b'e',
		b'n',
		b's',
		b'i',
		b'o',
		b'n',
		Type::False.into(),
		Type::MapEnd.into(),
	])
}
