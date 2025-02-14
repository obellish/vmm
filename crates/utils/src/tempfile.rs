use std::{
	env::temp_dir,
	ffi::{CString, OsStr},
	fs::{self, File},
	path::{Path, PathBuf},
};

use super::errno::{Error, Result, errno_result};

#[derive(Debug)]
pub struct TempFile {
	path: PathBuf,
	file: Option<File>,
}

impl TempFile {
	pub fn with_prefix(prefix: impl AsRef<OsStr>) -> Result<Self> {}
}
