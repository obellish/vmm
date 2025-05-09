#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ShortName<'a>(pub &'a str);

impl<'a> ShortName<'a> {
	#[must_use]
	pub const fn into_inner(self) -> &'a str {
		self.0
	}
}

impl Debug for ShortName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let full_name = self.into_inner();

		let mut index = 0usize;
		let end_of_string = full_name.len();

		while index < end_of_string {
			let rest_of_string = full_name.get(index..end_of_string).unwrap_or_default();

			if let Some(special_character_index) =
				rest_of_string.find([' ', '<', '>', '(', ')', '[', ']', ',', ';'])
			{
				let segments_to_collapse = rest_of_string
					.get(0..special_character_index)
					.unwrap_or_default();

				f.write_str(collapse_type_name(segments_to_collapse))?;

				let special_character =
					&rest_of_string[special_character_index..=special_character_index];

				f.write_str(special_character)?;

				match special_character {
					">" | ")" | "]"
						if rest_of_string[special_character_index + 1..].starts_with("::") =>
					{
						f.write_str("::")?;

						index += special_character_index + 3;
					}
					_ => index += special_character_index + 1,
				}
			} else {
				f.write_str(collapse_type_name(rest_of_string))?;
				index = end_of_string;
			}
		}

		Ok(())
	}
}

impl Display for ShortName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self, f)
	}
}

impl<'a> From<&'a str> for ShortName<'a> {
	fn from(value: &'a str) -> Self {
		Self(value)
	}
}

fn collapse_type_name(s: &str) -> &str {
	let mut segments = s.rsplit("::");
	let (last, second_last) = (segments.next().unwrap(), segments.next());
	let Some(second_last) = second_last else {
		return last;
	};

	if second_last.starts_with(char::is_uppercase) {
		let index = s.len() - last.len() - second_last.len() - 2;
		&s[index..]
	} else {
		last
	}
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
	use alloc::string::ToString;

	use super::ShortName;

	#[test]
	fn trivial() {
		assert_eq!(ShortName("test_system").to_string(), "test_system");
	}

	#[test]
	fn path_separated() {
		assert_eq!(
			ShortName("bevy_prelude::make_fun_game").to_string(),
			"make_fun_game"
		);
	}

	#[test]
	fn tuple_type() {
		assert_eq!(
			ShortName("(String, String)").to_string(),
			"(String, String)"
		);
	}

	#[test]
	fn array_type() {
		assert_eq!(ShortName("[i32; 3]").to_string(), "[i32; 3]");
	}

	#[test]
	fn trivial_generics() {
		assert_eq!(ShortName("a<B>").to_string(), "a<B>");
	}

	#[test]
	fn multiple_type_parameters() {
		assert_eq!(ShortName("a<B, C>").to_string(), "a<B, C>");
	}

	#[test]
	fn enums() {
		assert_eq!(ShortName("Option::None").to_string(), "Option::None");
		assert_eq!(ShortName("Option::Some(2)").to_string(), "Option::Some(2)");
	}
}
