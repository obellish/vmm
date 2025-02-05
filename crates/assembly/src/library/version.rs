use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	str::{self, FromStr},
};

use thiserror::Error;

use crate::{
	diagnostics::Diagnostic,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
	pub major: u16,
	pub minor: u16,
	pub patch: u16,
}

impl Version {
	#[must_use]
	pub const fn min() -> Self {
		Self {
			major: 0,
			minor: 1,
			patch: 0,
		}
	}

	#[must_use]
	pub const fn to_nearest_major(self) -> Self {
		Self {
			minor: 0,
			patch: 0,
			..self
		}
	}

	#[must_use]
	pub const fn to_nearest_minor(self) -> Self {
		Self { patch: 0, ..self }
	}

	#[must_use]
	pub const fn next_major(self) -> Self {
		Self {
			major: self.major + 1,
			minor: 0,
			patch: 0,
		}
	}

	#[must_use]
	pub const fn next_minor(self) -> Self {
		Self {
			minor: self.minor + 1,
			patch: 0,
			..self
		}
	}

	#[must_use]
	pub const fn next_patch(self) -> Self {
		Self {
			patch: self.patch + 1,
			..self
		}
	}
}

impl Default for Version {
	fn default() -> Self {
		Self::min()
	}
}

impl Deserializable for Version {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let major = source.read_u16()?;
		let minor = source.read_u16()?;
		let patch = source.read_u16()?;

		Ok(Self {
			major,
			minor,
			patch,
		})
	}
}

impl Display for Version {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.major, f)?;
		f.write_char('.')?;
		Display::fmt(&self.minor, f)?;
		f.write_char('.')?;
		Display::fmt(&self.patch, f)
	}
}

impl FromStr for Version {
	type Err = VersionError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut components = s.split('.');

		let major = components
			.next()
			.ok_or(VersionError::Empty)?
			.parse()
			.map_err(VersionError::Major)?;
		let minor = components
			.next()
			.ok_or(VersionError::MissingMinor)?
			.parse()
			.map_err(VersionError::Minor)?;
		let patch = components
			.next()
			.ok_or(VersionError::MissingPatch)?
			.parse()
			.map_err(VersionError::Patch)?;

		if components.next().is_some() {
			Err(VersionError::Unsupported)
		} else {
			Ok(Self {
				major,
				minor,
				patch,
			})
		}
	}
}

impl Serializable for Version {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u16(self.major);
		target.write_u16(self.minor);
		target.write_u16(self.patch);
	}

	fn get_size_hint(&self) -> usize {
		6
	}
}

#[derive(Debug, Error, Diagnostic)]
pub enum VersionError {
	#[error("invalid version string: cannot be empty")]
	#[diagnostic()]
	Empty,
	#[error("invalid version string: missing minor component, expected MAJOR.MINOR.PATCH")]
	#[diagnostic()]
	MissingMinor,
	#[error("invalid version string: missing patch component, expected MAJOR.MINOR.PATCH")]
	#[diagnostic()]
	MissingPatch,
	#[error("invalid version string: could not parse major version: {0}")]
	#[diagnostic()]
	Major(#[source] core::num::ParseIntError),
	#[error("invalid version string: could not parse minor version: {0}")]
	#[diagnostic()]
	Minor(#[source] core::num::ParseIntError),
	#[error("invalid version string: could not parse patch version: {0}")]
	#[diagnostic()]
	Patch(#[source] core::num::ParseIntError),
	#[error(
		"invalid version string: unsupported pre-release version, \
        only MAJOR.MINOR.PATCH components are allowed"
	)]
	#[diagnostic()]
	Unsupported,
}
