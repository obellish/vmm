use bevy::{
	app::{PluginGroup, PluginGroupBuilder},
	dev_tools::fps_overlay::FpsOverlayPlugin,
	diagnostic::{
		EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
	},
};

pub struct DebugPlugins;

impl PluginGroup for DebugPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(FpsOverlayPlugin::default())
			.add(LogDiagnosticsPlugin::default())
			.add(EntityCountDiagnosticsPlugin)
			.add(SystemInformationDiagnosticsPlugin)
	}
}
