use std::{io::Error as IoError, num::TryFromIntError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComponentCreationError {
	#[error("buffered capacity must not exceed your CPU architecture (e.g. 32-bit size)")]
	MustNotExceedArchitectureSize(#[from] TryFromIntError),
	#[error("buffered capacity cannot be zero")]
	CapacityCannotBeZero,
	#[error("buffered capacity must be aligned")]
	CapacityMustBeAligned,
	#[error("size cannot be lower than the initial size")]
	CannotBeLowerThanInitialSize,
	#[error(transparent)]
	Io(#[from] IoError),
}
