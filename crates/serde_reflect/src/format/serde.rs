use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, Visitor},
};

use super::Format;

impl<'de> Deserialize<'de> for Format {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		todo!()
	}
}

impl Serialize for Format {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Variable(v) => super::not_implemented::serialize(v, serializer),
			Self::TypeName(s) => serializer.serialize_newtype_variant("Format", 1, "TYPENAME", s),
			Self::Unit => serializer.serialize_unit_variant("Format", 1, "UNIT"),
			Self::Bool => serializer.serialize_unit_variant("Format", 2, "BOOL"),

			_ => todo!(),
		}
	}
}
