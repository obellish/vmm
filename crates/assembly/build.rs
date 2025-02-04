extern crate alloc;
extern crate lalrpop;

use alloc::boxed::Box;
use core::error::Error as StdError;

use rustc_version::{Channel, version_meta};

fn main() -> Result<(), Box<dyn StdError>> {
	lalrpop::process_root()?;

	println!("cargo:rustc-check-cfg=cfg(nightly)");
	if matches!(version_meta()?.channel, Channel::Nightly) {
		println!("cargo:rustc-cfg=nightly");
	}

	Ok(())
}
