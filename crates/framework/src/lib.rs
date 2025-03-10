#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod prelude;
pub mod save;

use bevy_app::{PluginGroup, PluginGroupBuilder};
pub use bevy_framework_check as check;
pub use bevy_framework_kind as kind;
pub use bevy_framework_object as object;
pub use bevy_framework_save::load;
pub use bevy_framework_tag as tag;
pub use bevy_framework_utils as utils;

pub struct BevyFrameworkPlugins;

impl PluginGroup for BevyFrameworkPlugins {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(self::save::SavePlugin)
			.add(self::load::LoadPlugin)
			.add(self::tag::TagPlugin)
	}
}
