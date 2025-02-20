use bevy::{log::Level, prelude::*};
use bevy_egui::egui::{Color32, FontId, TextFormat};

#[derive(Debug, Reflect)]
pub struct ConsoleTheme {
	#[reflect(ignore)]
	pub font: FontId,
	pub text_color: Color,
	pub dark: Color,
	pub error: Color,
	pub warning: Color,
	pub info: Color,
	pub debug: Color,
	pub trace: Color,
}

pub trait ToColor32 {
	fn to_color32(&self) -> Color32;
}

impl ToColor32 for Color {
	fn to_color32(&self) -> Color32 {
		let Srgba {
			red,
			green,
			blue,
			alpha,
		} = self.to_srgba();

		Color32::from_rgba_unmultiplied(
			(red * 255.0) as u8,
			(green * 255.0) as u8,
			(blue * 255.0) as u8,
			(alpha * 255.0) as u8,
		)
	}
}

macro_rules! define_text_format_method {
	($name:ident, $color:ident) => {
		pub fn $name(&self) -> ::bevy_egui::egui::TextFormat {
			use $crate::config::ToColor32 as _;

			::bevy_egui::egui::TextFormat {
				color: self.$color.to_color32(),
				..self.format_text()
			}
		}
	};
}
