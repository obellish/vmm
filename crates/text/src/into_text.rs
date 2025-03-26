use std::borrow::Cow;

use super::{ClickEvent, Color, Font, HoverEvent, Text};

pub trait IntoText<'a>: Sized {
	fn into_cow_text(self) -> Cow<'a, Text>;

	fn into_text(self) -> Text {
		self.into_cow_text().into_owned()
	}

	fn color(self, color: impl Into<Color>) -> Text {
		let mut value = self.into_text();
		value.color = Some(color.into());
		value
	}

	fn clear_color(self) -> Text {
		let mut value = self.into_text();
		value.color = None;
		value
	}

	fn font(self, font: Font) -> Text {
		let mut value = self.into_text();
		value.font = Some(font);
		value
	}

	fn clear_font(self) -> Text {
		let mut value = self.into_text();
		value.font = None;
		value
	}

	fn bold(self) -> Text {
		let mut value = self.into_text();
		value.bold = Some(true);
		value
	}

	fn not_bold(self) -> Text {
		let mut value = self.into_text();
		value.bold = Some(false);
		value
	}

	fn clear_bold(self) -> Text {
		let mut value = self.into_text();
		value.bold = None;
		value
	}

	fn italic(self) -> Text {
		let mut value = self.into_text();
		value.italic = Some(true);
		value
	}

	fn not_italic(self) -> Text {
		let mut value = self.into_text();
		value.italic = Some(false);
		value
	}

	fn clear_italic(self) -> Text {
		let mut value = self.into_text();
		value.italic = None;
		value
	}

	fn underlined(self) -> Text {
		let mut value = self.into_text();
		value.underlined = Some(true);
		value
	}

	fn not_underlined(self) -> Text {
		let mut value = self.into_text();
		value.underlined = Some(false);
		value
	}

	fn clear_underlined(self) -> Text {
		let mut value = self.into_text();
		value.underlined = None;
		value
	}

	fn strikethrough(self) -> Text {
		let mut value = self.into_text();
		value.strikethrough = Some(true);
		value
	}

	fn not_strikethrough(self) -> Text {
		let mut value = self.into_text();
		value.strikethrough = Some(false);
		value
	}

	fn clear_strikethrough(self) -> Text {
		let mut value = self.into_text();
		value.strikethrough = None;
		value
	}

	fn obfuscated(self) -> Text {
		let mut value = self.into_text();
		value.obfuscated = Some(true);
		value
	}

	fn not_obfuscated(self) -> Text {
		let mut value = self.into_text();
		value.obfuscated = Some(false);
		value
	}

	fn clear_obfuscated(self) -> Text {
		let mut value = self.into_text();
		value.obfuscated = None;
		value
	}

	fn insertion(self, insertion: impl Into<Cow<'static, str>>) -> Text {
		let mut value = self.into_text();
		value.insertion = Some(insertion.into());
		value
	}

	fn clear_insertion(self) -> Text {
		let mut value = self.into_text();
		value.insertion = None;
		value
	}

	fn on_click_open_url(self, url: impl Into<Cow<'static, str>>) -> Text {
		let mut value = self.into_text();
		value.click_event = Some(ClickEvent::OpenUrl(url.into()));
		value
	}

	fn on_click_run_command(self, command: impl Into<Cow<'static, str>>) -> Text {
		let mut value = self.into_text();
		value.click_event = Some(ClickEvent::RunCommand(command.into()));
		value
	}

	fn on_click_suggest_command(self, command: impl Into<Cow<'static, str>>) -> Text {
		let mut value = self.into_text();
		value.click_event = Some(ClickEvent::SuggestCommand(command.into()));
		value
	}

	fn on_click_change_page(self, page: impl Into<i32>) -> Text {
		let mut value = self.into_text();
		value.click_event = Some(ClickEvent::ChangePage(page.into()));
		value
	}

	fn on_click_copy_to_clipboard(self, text: impl Into<Cow<'static, str>>) -> Text {
		let mut value = self.into_text();
		value.click_event = Some(ClickEvent::CopyToClipboard(text.into()));
		value
	}

	fn clear_click_event(self) -> Text {
		let mut value = self.into_text();
		value.click_event = None;
		value
	}

	fn clear_hover_event(self) -> Text {
		let mut value = self.into_text();
		value.hover_event = None;
		value
	}

	fn on_hover_show_text(self, text: impl IntoText<'static>) -> Text {
		let mut value = self.into_text();
		value.hover_event = Some(HoverEvent::ShowText(text.into_text()));
		value
	}

	fn add_child(self, text: impl IntoText<'static>) -> Text {
		let mut value = self.into_text();
		value.extra.push(text.into_text());
		value
	}
}

impl<'a> IntoText<'a> for Text {
	fn into_cow_text(self) -> Cow<'a, Text> {
		Cow::Owned(self)
	}
}

impl<'a> IntoText<'a> for &'a Text {
	fn into_cow_text(self) -> Cow<'a, Text> {
		Cow::Borrowed(self)
	}
}

impl<'a> IntoText<'a> for Cow<'a, Text> {
	fn into_cow_text(self) -> Self {
		self
	}
}

impl<'a> IntoText<'a> for &'a Cow<'_, Text> {
	fn into_cow_text(self) -> Cow<'a, Text> {
		self.clone()
	}
}

impl<'a> IntoText<'a> for String {
	fn into_cow_text(self) -> Cow<'a, Text> {
		Cow::Owned(Text::text(self))
	}
}

impl<'b> IntoText<'b> for &String {
	fn into_cow_text(self) -> Cow<'b, Text> {
		Cow::Owned(Text::text(self.clone()))
	}
}

impl<'a> IntoText<'a> for Cow<'static, str> {
	fn into_cow_text(self) -> Cow<'a, Text> {
		Cow::Owned(Text::text(self))
	}
}

impl IntoText<'static> for &Cow<'static, str> {
	fn into_cow_text(self) -> Cow<'static, Text> {
		Cow::Owned(Text::text(self.clone()))
	}
}

impl<'a> IntoText<'a> for &'static str {
	fn into_cow_text(self) -> Cow<'a, Text> {
		Cow::Owned(Text::text(self))
	}
}

impl<'a, 'b, T, const N: usize> IntoText<'b> for [T; N]
where
	T: IntoText<'a>,
{
	fn into_cow_text(self) -> Cow<'b, Text> {
		let mut txt = Text::text("");

		for child in self {
			txt = txt.add_child(child.into_text());
		}

		Cow::Owned(txt)
	}
}

impl<'a, 'c, T, const N: usize> IntoText<'c> for &[T; N]
where
	T: Clone + IntoText<'a>,
{
	fn into_cow_text(self) -> Cow<'c, Text> {
		let mut txt = Text::text("");

		for child in self {
			txt = txt.add_child(child.clone().into_text());
		}

		Cow::Owned(txt)
	}
}

macro_rules! impl_primitives {
    ($($primitive:ty),+) => {
        $(
            impl<'a> $crate::IntoText<'a> for $primitive {
                fn into_cow_text(self) -> ::std::borrow::Cow<'a, $crate::Text> {
                    use ::std::string::ToString as _;

                    ::std::borrow::Cow::Owned($crate::Text::text(self.to_string()))
                }
            }
        )+
    };
}

impl_primitives! { char, bool, f32, f64, isize, usize, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128 }

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use crate::IntoText;

	fn is_borrowed<'a>(value: impl IntoText<'a>) -> bool {
		matches!(value.into_cow_text(), Cow::Borrowed(..))
	}

	#[test]
	#[expect(clippy::needless_borrows_for_generic_args)]
	fn into_text() {
		assert!(is_borrowed(&"this should be borrowed".into_text()));
		assert!(is_borrowed(&"this should be borrowed too".bold()));
		assert!(!is_borrowed("this should be owned?".bold()));
		assert!(!is_borrowed("this should be owned"));
		assert!(!is_borrowed(465));
		assert!(!is_borrowed(false));
	}
}
