use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RgbColor {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl RgbColor {
	#[must_use]
	pub const fn new(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b }
	}

	#[must_use]
	pub fn to_named_lossy(self) -> NamedColor {
		[
			NamedColor::Aqua,
			NamedColor::Black,
			NamedColor::Blue,
			NamedColor::DarkAqua,
			NamedColor::DarkBlue,
			NamedColor::DarkGray,
			NamedColor::DarkGreen,
			NamedColor::DarkPurple,
			NamedColor::DarkRed,
			NamedColor::Gold,
			NamedColor::Gray,
			NamedColor::Green,
			NamedColor::LightPurple,
			NamedColor::Red,
			NamedColor::White,
			NamedColor::Yellow,
		]
		.into_iter()
		.min_by_key(|&named| Self::from(named).squared_distance(self))
		.unwrap()
	}

	fn squared_distance(self, c2: Self) -> i32 {
		(i32::from(self.r) - i32::from(c2.r)).pow(2)
			+ (i32::from(self.g) - i32::from(c2.g)).pow(2)
			+ (i32::from(self.b) - i32::from(c2.b)).pow(2)
	}
}

impl Display for RgbColor {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
	}
}

impl From<NamedColor> for RgbColor {
	fn from(value: NamedColor) -> Self {
		match value {
			NamedColor::Aqua => Self::new(85, 255, 255),
			NamedColor::Black => Self::new(0, 0, 0),
			NamedColor::Blue => Self::new(85, 85, 255),
			NamedColor::DarkAqua => Self::new(0, 170, 170),
			NamedColor::DarkBlue => Self::new(0, 0, 170),
			NamedColor::DarkGray => Self::new(85, 85, 85),
			NamedColor::DarkGreen => Self::new(0, 170, 0),
			NamedColor::DarkPurple => Self::new(170, 0, 170),
			NamedColor::DarkRed => Self::new(170, 0, 0),
			NamedColor::Gold => Self::new(255, 170, 0),
			NamedColor::Gray => Self::new(170, 170, 170),
			NamedColor::Green => Self::new(85, 255, 85),
			NamedColor::LightPurple => Self::new(255, 85, 255),
			NamedColor::Red => Self::new(255, 85, 85),
			NamedColor::White => Self::new(255, 255, 255),
			NamedColor::Yellow => Self::new(255, 255, 85),
		}
	}
}

impl FromStr for RgbColor {
	type Err = ColorError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let to_num = |d| match d {
			b'0'..=b'9' => Ok(d - b'0'),
			b'a'..=b'f' => Ok(d - b'a' + 0xa),
			b'A'..=b'F' => Ok(d - b'A' + 0xa),
			_ => Err(ColorError),
		};

		if let &[b'#', r0, r1, g0, g1, b0, b1] = s.as_bytes() {
			Ok(Self {
				r: (to_num(r0)? << 4) | to_num(r1)?,
				g: (to_num(g0)? << 4) | to_num(g1)?,
				b: (to_num(b0)? << 4) | to_num(b1)?,
			})
		} else {
			Err(ColorError)
		}
	}
}

impl TryFrom<&str> for RgbColor {
	type Error = ColorError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		value.parse()
	}
}

struct ColorVisitor;

impl Visitor<'_> for ColorVisitor {
	type Value = Color;

	fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("a hex color (#rrggbb), a normal color or 'reset'")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Color::try_from(v).map_err(|_| E::custom("invalid color"))
	}
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialOrd, Ord)]
pub enum Color {
	#[default]
	Reset,
	Rgb(RgbColor),
	Named(NamedColor),
}

impl Color {
	pub const AQUA: Self = Self::Named(NamedColor::Aqua);
	pub const BLACK: Self = Self::Named(NamedColor::Black);
	pub const BLUE: Self = Self::Named(NamedColor::Blue);
	pub const DARK_AQUA: Self = Self::Named(NamedColor::DarkAqua);
	pub const DARK_BLUE: Self = Self::Named(NamedColor::DarkBlue);
	pub const DARK_GRAY: Self = Self::Named(NamedColor::DarkGray);
	pub const DARK_GREEN: Self = Self::Named(NamedColor::DarkGreen);
	pub const DARK_PURPLE: Self = Self::Named(NamedColor::DarkPurple);
	pub const DARK_RED: Self = Self::Named(NamedColor::DarkRed);
	pub const GOLD: Self = Self::Named(NamedColor::Gold);
	pub const GRAY: Self = Self::Named(NamedColor::Gray);
	pub const GREEN: Self = Self::Named(NamedColor::Green);
	pub const LIGHT_PURPLE: Self = Self::Named(NamedColor::LightPurple);
	pub const RED: Self = Self::Named(NamedColor::Red);
	pub const RESET: Self = Self::Reset;
	pub const WHITE: Self = Self::Named(NamedColor::White);
	pub const YELLOW: Self = Self::Named(NamedColor::Yellow);

	#[must_use]
	pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self::Rgb(RgbColor::new(r, g, b))
	}
}

impl<'de> Deserialize<'de> for Color {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(ColorVisitor)
	}
}

impl Display for Color {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Reset => f.write_str("reset"),
			Self::Rgb(rgb) => Display::fmt(&rgb, f),
			Self::Named(named) => Display::fmt(&named, f),
		}
	}
}

impl From<RgbColor> for Color {
	fn from(value: RgbColor) -> Self {
		Self::Rgb(value)
	}
}

impl From<NamedColor> for Color {
	fn from(value: NamedColor) -> Self {
		Self::Named(value)
	}
}

impl FromStr for Color {
	type Err = ColorError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with('#') {
			return Ok(Self::Rgb(s.parse()?));
		}

		if matches!(s, "reset") {
			return Ok(Self::Reset);
		}

		Ok(Self::Named(s.parse()?))
	}
}

impl Hash for Color {
	fn hash<H: Hasher>(&self, state: &mut H) {
		std::mem::discriminant(self).hash(state);

		match self {
			Self::Reset => {}
			Self::Rgb(rgb) => rgb.hash(state),
			Self::Named(named) => named.hash(state),
		}
	}
}

impl PartialEq for Color {
	fn eq(&self, other: &Self) -> bool {
		match (*self, *other) {
			(Self::Reset, Self::Reset) => true,
			(Self::Rgb(l), Self::Rgb(r)) => l == r,
			(Self::Named(l), Self::Named(r)) => l == r,
			(Self::Rgb(rgb), Self::Named(normal)) | (Self::Named(normal), Self::Rgb(rgb)) => {
				rgb == RgbColor::from(normal)
			}
			(Self::Reset, _) | (_, Self::Reset) => false,
		}
	}
}

impl Serialize for Color {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		format!("{self}").serialize(serializer)
	}
}

impl TryFrom<&str> for Color {
	type Error = ColorError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		value.parse()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamedColor {
	Black = 0,
	DarkBlue,
	DarkGreen,
	DarkAqua,
	DarkRed,
	DarkPurple,
	Gold,
	Gray,
	DarkGray,
	Blue,
	Green,
	Aqua,
	Red,
	LightPurple,
	Yellow,
	White,
}

impl NamedColor {
	#[must_use]
	pub const fn hex_digit(self) -> char {
		b"0123456789abcdef"[self as usize] as char
	}

	#[must_use]
	pub const fn name(self) -> &'static str {
		[
			"black",
			"dark_blue",
			"dark_green",
			"dark_aqua",
			"dark_red",
			"dark_purple",
			"gold",
			"gray",
			"dark_gray",
			"blue",
			"green",
			"aqua",
			"red",
			"light_purple",
			"yellow",
			"white",
		][self as usize]
	}
}

impl Display for NamedColor {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self.name())
	}
}

impl FromStr for NamedColor {
	type Err = ColorError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"black" => Ok(Self::Black),
			"dark_blue" => Ok(Self::DarkBlue),
			"dark_green" => Ok(Self::DarkGreen),
			"dark_aqua" => Ok(Self::DarkAqua),
			"dark_red" => Ok(Self::DarkRed),
			"dark_purple" => Ok(Self::DarkPurple),
			"gold" => Ok(Self::Gold),
			"gray" => Ok(Self::Gray),
			"dark_gray" => Ok(Self::DarkGray),
			"blue" => Ok(Self::Blue),
			"green" => Ok(Self::Green),
			"aqua" => Ok(Self::Aqua),
			"red" => Ok(Self::Red),
			"light_purple" => Ok(Self::LightPurple),
			"yellow" => Ok(Self::Yellow),
			"white" => Ok(Self::White),
			_ => Err(ColorError),
		}
	}
}

impl TryFrom<&str> for NamedColor {
	type Error = ColorError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		value.parse()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorError;

impl Display for ColorError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid color name or hex code")
	}
}

impl StdError for ColorError {}

#[cfg(test)]
mod tests {
	use super::{Color, NamedColor, RgbColor};

	#[test]
	fn colors() {
		assert_eq!(
			"#aBcDeF".parse::<RgbColor>(),
			Ok(RgbColor::new(0xab, 0xcd, 0xef))
		);

		assert_eq!(
			"#fFfFfF".parse::<RgbColor>(),
			Ok(RgbColor::new(255, 255, 255))
		);

		assert_eq!("#000000".parse::<Color>(), Ok(NamedColor::Black.into()));
		assert_eq!("red".parse::<Color>(), Ok(NamedColor::Red.into()));
		assert_eq!("blue".parse::<Color>(), Ok(NamedColor::Blue.into()));
		assert!("#ffTf00".parse::<Color>().is_err());
		assert!("#ffš00".parse::<Color>().is_err());
		assert!("#00000000".parse::<Color>().is_err());
		assert!("#".parse::<Color>().is_err());
	}
}
