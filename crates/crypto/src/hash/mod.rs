pub mod blake;
mod rescue;
pub mod rpo {
	pub use super::rescue::{Rpo256, RpoDigest, RpoDigestError};
}
pub mod rpx {
	pub use super::rescue::{Rpx256, RpxDigest, RpxDigestError};
}

pub use winter_crypto::{Digest, ElementHasher, Hasher};
