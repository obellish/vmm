#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind as IoErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
	mem,
	num::Wrapping,
};
