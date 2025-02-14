use std::{
	env::temp_dir,
	ffi::OsStr,
	fs::{self, File},
	path::{Path, PathBuf},
};
#[cfg(unix)]
use std::{
	ffi::CString,
	os::unix::{ffi::OsStrExt as _, io::FromRawFd as _},
};

use super::errno::{Error, Result, errno_result};

#[derive(Debug)]
pub struct TempFile {
	path: PathBuf,
	file: Option<File>,
}

impl TempFile {
	#[cfg(unix)]
	pub fn with_prefix(prefix: impl AsRef<OsStr>) -> Result<Self> {
		let mut os_fname = prefix.as_ref().to_os_string();
		os_fname.push("XXXXXX");

		let c_tempname = CString::new(os_fname.as_bytes()).map_err(|_| Error::new(libc::EINVAL))?;
		let raw_tempname = c_tempname.into_raw();

		let ret = unsafe { libc::mkstemp(raw_tempname) };

		let c_tempname = unsafe { CString::from_raw(raw_tempname) };

		let fd = match ret {
			-1 => return errno_result(),
			_ => ret,
		};

		let os_tempname = OsStr::from_bytes(c_tempname.as_bytes());

		let file = unsafe { File::from_raw_fd(fd) };

		Ok(Self {
			path: PathBuf::from(os_tempname),
			file: Some(file),
		})
	}
}
