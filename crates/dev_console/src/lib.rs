#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod config;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct DevConsolePlugin;

impl Plugin for DevConsolePlugin {
	fn build(&self, app: &mut App) {
		if !app.is_plugin_added::<EguiPlugin>() {
			app.add_plugins(EguiPlugin);
		}
	}
}
