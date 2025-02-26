use bevy::prelude::*;
use vmm::MainPlugin;

fn main() {
	App::new().add_plugins(MainPlugin).run();
}
