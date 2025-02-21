#[cfg(not(feature = "debug"))]
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::{
	app::{PluginGroup, PluginGroupBuilder},
	diagnostic::{
		EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
		SystemInformationDiagnosticsPlugin,
	},
};

pub struct DiagnosticsPlugin;

#[cfg(feature = "debug")]
impl PluginGroup for DiagnosticsPlugin {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(FrameTimeDiagnosticsPlugin)
			.add(LogDiagnosticsPlugin::default())
			.add(EntityCountDiagnosticsPlugin)
			.add(SystemInformationDiagnosticsPlugin)
	}
}

#[cfg(not(feature = "debug"))]
impl Plugin for DiagnosticsPlugin {
	fn build(&self, app: &mut App) {
		info!("not build with `debug` plugin, skipping diagnostics");
	}
}
