#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod color;
mod into_text;
#[cfg(test)]
mod tests;

use std::{
	borrow::Cow,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	ops::{Add, AddAssign, Deref, DerefMut},
	str::FromStr,
};

use serde::{
	Deserialize, Deserializer, Serialize,
	de::{Error as DeError, MapAccess, SeqAccess, Visitor, value::MapAccessDeserializer},
};
use uuid::Uuid;
use vmm_ident::Ident;
use vmm_nbt::Value;

#[doc(inline)]
pub use self::{color::Color, into_text::IntoText};

#[derive(Default, Clone, PartialEq, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Text(Box<TextInner>);

#[allow(clippy::self_named_constructors)]
impl Text {
	pub fn text(plain: impl Into<Cow<'static, str>>) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::Text { text: plain.into() },
			..Default::default()
		}))
	}

	pub fn translate(key: impl Into<Cow<'static, str>>, with: impl Into<Vec<Self>>) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::Translate {
				translate: key.into(),
				with: with.into(),
			},
			..Default::default()
		}))
	}

	pub fn score(
		name: impl Into<Cow<'static, str>>,
		objective: impl Into<Cow<'static, str>>,
		value: Option<Cow<'static, str>>,
	) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::ScoreboardValue {
				score: ScoreboardValueContent {
					name: name.into(),
					objective: objective.into(),
					value,
				},
			},
			..Default::default()
		}))
	}

	pub fn selector(selector: impl Into<Cow<'static, str>>, separator: Option<Self>) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::EntityNames {
				selector: selector.into(),
				separator,
			},
			..Default::default()
		}))
	}

	pub fn keybind(keybind: impl Into<Cow<'static, str>>) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::Keybind {
				keybind: keybind.into(),
			},
			..Default::default()
		}))
	}

	pub fn block_nbt(
		block: impl Into<Cow<'static, str>>,
		nbt: impl Into<Cow<'static, str>>,
		interpret: Option<bool>,
		separator: Option<Self>,
	) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::BlockNbt {
				block: block.into(),
				nbt: nbt.into(),
				interpret,
				separator,
			},
			..Default::default()
		}))
	}

	pub fn entity_nbt(
		entity: impl Into<Cow<'static, str>>,
		nbt: impl Into<Cow<'static, str>>,
		interpret: Option<bool>,
		separator: Option<Self>,
	) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::EntityNbt {
				entity: entity.into(),
				nbt: nbt.into(),
				interpret,
				separator,
			},
			..Default::default()
		}))
	}

	pub fn storage_nbt(
		storage: impl Into<Ident<Cow<'static, str>>>,
		nbt: impl Into<Cow<'static, str>>,
		interpret: Option<bool>,
		separator: Option<Self>,
	) -> Self {
		Self(Box::new(TextInner {
			content: TextContent::StorageNbt {
				storage: storage.into(),
				nbt: nbt.into(),
				interpret,
				separator,
			},
			..Default::default()
		}))
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		for extra in &self.0.extra {
			if !extra.is_empty() {
				return false;
			}
		}

		match &self.0.content {
			TextContent::Text { text } => text.is_empty(),
			TextContent::Translate { translate, .. } => translate.is_empty(),
			TextContent::ScoreboardValue {
				score: ScoreboardValueContent {
					name, objective, ..
				},
			} => name.is_empty() || objective.is_empty(),
			TextContent::EntityNames { selector, .. } => selector.is_empty(),
			TextContent::Keybind { keybind } => keybind.is_empty(),
			TextContent::BlockNbt { nbt, .. }
			| TextContent::EntityNbt { nbt, .. }
			| TextContent::StorageNbt { nbt, .. } => nbt.is_empty(),
		}
	}

	#[must_use]
	pub fn to_legacy_lossy(&self) -> String {
		#[derive(Default, Clone)]
		struct Modifiers {
			obfuscated: Option<bool>,
			bold: Option<bool>,
			strikethrough: Option<bool>,
			underlined: Option<bool>,
			italic: Option<bool>,
			color: Option<Color>,
		}

		impl Modifiers {
			fn write(&self, output: &mut String) {
				if let Some(color) = self.color {
					let code = match color {
						Color::Rgb(rgb) => rgb.to_named_lossy().hex_digit(),
						Color::Named(normal) => normal.hex_digit(),
						Color::Reset => return,
					};

					output.push('§');
					output.push(code);
				}

				if self.obfuscated.is_some_and(|v| v) {
					output.push_str("§k");
				}

				if self.bold.is_some_and(|v| v) {
					output.push_str("§l");
				}

				if self.strikethrough.is_some_and(|v| v) {
					output.push_str("§m");
				}

				if self.underlined.is_some_and(|v| v) {
					output.push_str("§n");
				}

				if self.italic.is_some_and(|v| v) {
					output.push_str("§o");
				}
			}

			fn add(&self, other: &Self) -> Self {
				Self {
					obfuscated: other.obfuscated.or(self.obfuscated),
					bold: other.bold.or(self.bold),
					strikethrough: other.strikethrough.or(self.strikethrough),
					underlined: other.underlined.or(self.underlined),
					italic: other.italic.or(self.italic),
					color: other.color.or(self.color),
				}
			}
		}

		fn to_legacy_inner(this: &Text, result: &mut String, mods: &mut Modifiers) {
			let new_mods = Modifiers {
				obfuscated: this.0.obfuscated,
				bold: this.0.bold,
				strikethrough: this.0.strikethrough,
				underlined: this.0.underlined,
				italic: this.0.italic,
				color: this.0.color,
			};

			if [
				this.0.obfuscated,
				this.0.bold,
				this.0.strikethrough,
				this.0.underlined,
				this.0.italic,
			]
			.contains(&Some(false))
				|| matches!(this.0.color, Some(Color::Reset))
			{
				result.push_str("§r");
				mods.add(&new_mods).write(result);
			} else {
				new_mods.write(result);
			}

			*mods = mods.add(&new_mods);

			if let TextContent::Text { text } = &this.0.content {
				result.push_str(text);
			}

			for child in &this.0.extra {
				to_legacy_inner(child, result, mods);
			}
		}

		let mut result = String::new();
		let mut mods = Modifiers::default();

		to_legacy_inner(self, &mut result, &mut mods);

		result
	}
}

impl<T> Add<T> for Text
where
	T: IntoText<'static>,
{
	type Output = Self;

	fn add(self, rhs: T) -> Self::Output {
		self.add_child(rhs)
	}
}

impl<T> AddAssign<T> for Text
where
	T: IntoText<'static>,
{
	fn add_assign(&mut self, rhs: T) {
		self.extra.push(rhs.into_text());
	}
}

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Deref for Text {
	type Target = TextInner;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Text {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'de> Deserialize<'de> for Text {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(TextVisitor)
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let string = if f.alternate() {
			serde_json::to_string_pretty(self)
		} else {
			serde_json::to_string(self)
		}
		.map_err(|_| std::fmt::Error)?;

		f.write_str(&string)
	}
}

impl From<Text> for Cow<'_, Text> {
	fn from(value: Text) -> Self {
		Self::Owned(value)
	}
}

impl<'a> From<&'a Text> for Cow<'a, Text> {
	fn from(value: &'a Text) -> Self {
		Self::Borrowed(value)
	}
}

impl From<Text> for String {
	fn from(value: Text) -> Self {
		format!("{value}")
	}
}

impl From<Text> for Value {
	fn from(value: Text) -> Self {
		Self::String(value.into())
	}
}

impl<'a> From<&'a Self> for Text {
	fn from(value: &'a Self) -> Self {
		value.clone()
	}
}

impl<'a> From<Cow<'a, Self>> for Text {
	fn from(value: Cow<'a, Self>) -> Self {
		value.into_owned()
	}
}

impl<'a, 'b> From<&'a Cow<'b, Self>> for Text {
	fn from(value: &'a Cow<'b, Self>) -> Self {
		value.clone().into_owned()
	}
}

impl From<String> for Text {
	fn from(value: String) -> Self {
		value.into_text()
	}
}

impl<'a> From<&'a String> for Text {
	fn from(value: &'a String) -> Self {
		value.into_text()
	}
}

impl From<Cow<'static, str>> for Text {
	fn from(value: Cow<'static, str>) -> Self {
		value.into_text()
	}
}

impl<'a> From<&'a Cow<'static, str>> for Text {
	fn from(value: &'a Cow<'static, str>) -> Self {
		value.into_text()
	}
}

impl From<&'static str> for Text {
	fn from(value: &'static str) -> Self {
		value.into_text()
	}
}

impl FromStr for Text {
	type Err = serde_json::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.is_empty() {
			Ok(Self::default())
		} else {
			serde_json::from_str(s)
		}
	}
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextInner {
	#[serde(flatten)]
	pub content: TextContent,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub color: Option<Color>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub font: Option<Font>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub bold: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub italic: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub underlined: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub strikethrough: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub obfuscated: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub insertion: Option<Cow<'static, str>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub click_event: Option<ClickEvent>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hover_event: Option<HoverEvent>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub extra: Vec<Text>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreboardValueContent {
	pub name: Cow<'static, str>,
	pub objective: Cow<'static, str>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub value: Option<Cow<'static, str>>,
}

struct TextVisitor;

impl<'de> Visitor<'de> for TextVisitor {
	type Value = Text;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a text component data typeF")
	}

	fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Text::text(v.to_string()))
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Text::text(v.to_string()))
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Text::text(v.to_string()))
	}

	fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Text::text(v.to_string()))
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Text::text(v.to_owned()))
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Text::text(v))
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(mut res) = seq.next_element()? else {
			return Ok(Text::default());
		};

		while let Some(child) = seq.next_element::<Text>()? {
			res += child;
		}

		Ok(res)
	}

	fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		Ok(Text(Box::new(TextInner::deserialize(
			MapAccessDeserializer::new(map),
		)?)))
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextContent {
	Text {
		text: Cow<'static, str>,
	},
	Translate {
		translate: Cow<'static, str>,
		#[serde(default, skip_serializing_if = "Vec::is_empty")]
		with: Vec<Text>,
	},
	ScoreboardValue {
		score: ScoreboardValueContent,
	},
	EntityNames {
		selector: Cow<'static, str>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		separator: Option<Text>,
	},
	Keybind {
		keybind: Cow<'static, str>,
	},
	BlockNbt {
		block: Cow<'static, str>,
		nbt: Cow<'static, str>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		interpret: Option<bool>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		separator: Option<Text>,
	},
	EntityNbt {
		entity: Cow<'static, str>,
		nbt: Cow<'static, str>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		interpret: Option<bool>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		separator: Option<Text>,
	},
	StorageNbt {
		storage: Ident<Cow<'static, str>>,
		nbt: Cow<'static, str>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		interpret: Option<bool>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		separator: Option<Text>,
	},
}

impl Default for TextContent {
	fn default() -> Self {
		Self::Text {
			text: Cow::Borrowed(""),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "value", rename_all = "snake_case")]
pub enum ClickEvent {
	OpenUrl(Cow<'static, str>),
	OpenFile(Cow<'static, str>),
	RunCommand(Cow<'static, str>),
	SuggestCommand(Cow<'static, str>),
	ChangePage(i32),
	CopyToClipboard(Cow<'static, str>),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "contents", rename_all = "snake_case")]
pub enum HoverEvent {
	ShowText(Text),
	ShowItem {
		id: Ident<Cow<'static, str>>,
		count: Option<i32>,
		tag: Cow<'static, str>,
	},
	ShowEntity {
		id: Uuid,
		#[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
		kind: Option<Ident<Cow<'static, str>>>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		name: Option<Text>,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Font {
	#[serde(rename = "minecraft:default")]
	Default,
	#[serde(rename = "minecraft:uniform")]
	Uniform,
	#[serde(rename = "minecraft:alt")]
	Alt,
}
