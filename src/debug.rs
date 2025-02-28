#[cfg(not(feature = "debug"))]
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::{
	app::{PluginGroup, PluginGroupBuilder},
	dev_tools::fps_overlay::FpsOverlayPlugin,
	diagnostic::{
		EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
	},
};

pub struct DebugPlugins;

#[cfg(feature = "debug")]
impl PluginGroup for DebugPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(FpsOverlayPlugin::default())
			.add(LogDiagnosticsPlugin::default())
			.add(EntityCountDiagnosticsPlugin)
			.add(SystemInformationDiagnosticsPlugin)
	}
}

#[cfg(not(feature = "debug"))]
impl Plugin for DebugPlugins {
	fn build(&self, _: &mut App) {
		info!("not built with `debug` feature, skipping diagnostics");
	}
}
