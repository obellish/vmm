mod attributes;
mod block;
mod chunk_view;
mod item;
mod packet_id;
mod sound;
mod status_effects;

use anyhow::Result;
use vmm_build_utils::write_generated_file;

fn main() -> Result<()> {
	write_generated_file(self::attributes::build()?, "attributes.rs")?;
	write_generated_file(self::block::build()?, "block.rs")?;
	write_generated_file(self::item::build()?, "item.rs")?;
	write_generated_file(self::sound::build()?, "sound.rs")?;
	write_generated_file(self::packet_id::build()?, "packet_id.rs")?;
	write_generated_file(self::chunk_view::build(), "chunk_view.rs")?;
	write_generated_file(self::status_effects::build()?, "status_effects.rs")?;

	Ok(())
}
