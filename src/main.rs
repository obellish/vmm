use bevy::{
	prelude::*,
	remote::{RemotePlugin, http::RemoteHttpPlugin},
};
use vmm::debug::DiagnosticsPlugin;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins,
			RemotePlugin::default(),
			RemoteHttpPlugin::default(),
			DiagnosticsPlugin,
		))
		.run();
}
