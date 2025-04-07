#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::{mem, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Serialize};

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new("([a-zA-Z0-9§\\-:/]+\\.[a-zA-Z/0-9§\\-:_#]+(\\.[a-zA-Z/0-9.§\\-:#\\?\\+=_]+)?)")
		.unwrap()
});

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextComponent {
	pub text: String,
	#[serde(skip_serializing_if = "is_false")]
	pub bold: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub italic: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub underlined: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub strikethrough: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub obfuscated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub color: Option<TextColor>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "clickEvent")]
	pub click_event: Option<ClickEvent>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub extra: Vec<TextComponent>,
}

impl TextComponent {
	pub fn from_legacy_text(message: &str) -> Vec<Self> {
		let mut components = Vec::new();

		let mut cur_component = Self::default();

		let mut chars = message.chars();
		'main_loop: while let Some(c) = chars.next() {
			if matches!(c, '&') {
				if let Some(code) = chars.next() {
					if let Some(color) = ColorCode::parse(code) {
						let make_new = !cur_component.text.is_empty();
						if color.is_formatting() && make_new {
							components.push(cur_component.clone());
							cur_component.text.clear();
						}

						match color {
							ColorCode::Bold => cur_component.bold = true,
							ColorCode::Italic => cur_component.italic = true,
							ColorCode::Underline => cur_component.underlined = true,
							ColorCode::Strikethrough => cur_component.strikethrough = true,
							ColorCode::Obfuscated => cur_component.obfuscated = true,
							_ => {
								components.push(mem::take(&mut cur_component));
								cur_component.color = Some(TextColor::ColorCode(color));
							}
						}

						continue;
					}

					cur_component.text.push(c);
					cur_component.text.push(code);
					continue;
				}
			}

			if matches!(c, '#') {
				let mut hex = String::from(c);
				for _ in 0..6 {
					if let Some(c) = chars.next() {
						hex.push(c);
						if !is_valid_hex(c) {
							cur_component.text += &hex;
							continue 'main_loop;
						}
					} else {
						cur_component.text += &hex;
						continue 'main_loop;
					}
				}

				components.push(mem::take(&mut cur_component));
				cur_component.color = Some(TextColor::Hex(hex));
				continue;
			}

			cur_component.text.push(c);
		}

		components.push(cur_component);

		let mut new_components = Vec::with_capacity(components.len());

		for component in components {
			let mut last = 0;
			let text = &component.text;

			for match_ in URL_REGEX.find_iter(text) {
				let index = match_.start();
				let matched = match_.as_str();
				if last != index {
					let mut new = component.clone();
					new.text = String::from(&text[last..index]);
					new_components.push(new);
				}

				let mut new = component.clone();
				matched.clone_into(&mut new.text);
				new.click_event = Some(ClickEvent {
					action: ClickEventType::OpenUrl,
					value: matched.to_owned(),
				});
				new_components.push(new);
				last = index + matched.len();
			}

			if last < text.len() {
				let mut new = component.clone();
				new.text = String::from(&text[last..]);
				new_components.push(new);
			}
		}

		new_components
	}

	pub fn try_encode_json(&self) -> Result<String, serde_json::Error> {
		serde_json::to_string(self)
	}

	#[must_use]
	pub fn encode_json(&self) -> String {
		self.try_encode_json().unwrap()
	}

	#[must_use]
	pub fn is_text_only(&self) -> bool {
		!self.bold
			&& !self.italic
			&& !self.underlined
			&& !self.strikethrough
			&& !self.obfuscated
			&& self.extra.is_empty()
			&& self.color.is_none()
			&& self.click_event.is_none()
	}
}

impl<S> From<S> for TextComponent
where
	S: Into<String>,
{
	fn from(value: S) -> Self {
		Self {
			text: value.into(),
			..Default::default()
		}
	}
}

#[repr(transparent)]
pub struct TextComponentBuilder {
	component: TextComponent,
}

impl TextComponentBuilder {
	pub fn new(text: impl Into<String>) -> Self {
		let component = TextComponent {
			text: text.into(),
			..Default::default()
		};

		Self { component }
	}

	#[must_use]
	pub fn color(mut self, color: TextColor) -> Self {
		self.component.color = Some(color);
		self
	}

	#[must_use]
	pub fn color_code(self, color: ColorCode) -> Self {
		self.color(TextColor::ColorCode(color))
	}

	#[must_use]
	pub const fn strikethrough(mut self, value: bool) -> Self {
		self.component.strikethrough = value;
		self
	}

	#[must_use]
	pub fn finish(self) -> TextComponent {
		self.component
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent {
	action: ClickEventType,
	value: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorCode {
	Black,
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
	Obfuscated,
	Bold,
	Strikethrough,
	Underline,
	Italic,
	Reset,
}

impl ColorCode {
	const fn parse(code: char) -> Option<Self> {
		Some(match code {
			'0' => Self::Black,
			'1' => Self::DarkBlue,
			'2' => Self::DarkGreen,
			'3' => Self::DarkAqua,
			'4' => Self::DarkRed,
			'5' => Self::DarkPurple,
			'6' => Self::Gold,
			'7' => Self::Gray,
			'8' => Self::DarkGray,
			'9' => Self::Blue,
			'a' => Self::Green,
			'b' => Self::Aqua,
			'c' => Self::Red,
			'd' => Self::LightPurple,
			'e' => Self::Yellow,
			'f' => Self::White,
			'k' => Self::Obfuscated,
			'l' => Self::Bold,
			'm' => Self::Strikethrough,
			'n' => Self::Underline,
			'o' => Self::Italic,
			'r' => Self::Reset,
			_ => return None,
		})
	}

	const fn is_formatting(self) -> bool {
		matches!(
			self,
			Self::Obfuscated
				| Self::Bold | Self::Strikethrough
				| Self::Underline
				| Self::Italic
				| Self::Reset
		)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextColor {
	Hex(String),
	ColorCode(ColorCode),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ClickEventType {
	OpenUrl,
}

fn is_valid_hex(ch: char) -> bool {
	ch.is_numeric() || matches!(ch, 'a'..='f' | 'A'..='F')
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(field: &bool) -> bool {
	!*field
}
