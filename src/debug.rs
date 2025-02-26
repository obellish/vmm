#[cfg(not(feature = "debug"))]
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::{
	app::{PluginGroup, PluginGroupBuilder},
	dev_tools::{fps_overlay::FpsOverlayPlugin, picking_debug::DebugPickingPlugin},
	diagnostic::{
		EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
	},
};

pub struct DebugPlugins;

impl PluginGroup for DebugPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(FpsOverlayPlugin::default())
			.add(DebugPickingPlugin)
			.add(LogDiagnosticsPlugin::default())
			.add(EntityCountDiagnosticsPlugin)
			.add(SystemInformationDiagnosticsPlugin)
	}
}

#[cfg(not(feature = "debug"))]
impl Plugin for DebugPlugins {
	fn build(&self, app: &mut App) {
		info!("not built with `debug` feature, skipping diagnostics");
	}
}
