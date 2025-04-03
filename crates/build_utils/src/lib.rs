#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::{env, fs, path::Path, process::Command};

use anyhow::{Context as _, Result};
use proc_macro2::{Ident, Span, TokenStream};

pub fn write_generated_file(content: TokenStream, out_file: &str) -> Result<()> {
	let out_dir = env::var_os("OUT_DIR").context("failed to get OUT_DIR env var")?;
	let path = Path::new(&out_dir).join(out_file);
	let code = content.to_string();

	fs::write(&path, code)?;

	let _ = Command::new("rustfmt").arg(path).output();

	Ok(())
}

pub fn ident(s: impl AsRef<str>) -> Ident {
	let s = s.as_ref().trim();

	syn::parse_str::<Ident>(s)
		.unwrap_or_else(|_| Ident::new(format!("_{s}").as_str(), Span::call_site()))
}

#[track_caller]
pub fn rerun_if_changed<const N: usize>(files: [&str; N]) {
	for file in files {
		assert!(Path::new(file).exists(), "file \"{file}\" does not exist");

		println!("cargo:rerun-if-changed={file}");
	}
}
