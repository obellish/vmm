mod rpo;
mod rpx;

use rand::RngCore;
pub use winter_crypto::{DefaultRandomCoin as WinterRandomCoin, RandomCoin, RandomCoinError};
pub use winter_utils::Randomizable;

pub use self::{rpo::RpoRandomCoin, rpx::RpxRandomCoin};
use super::{Felt, Word};
use crate::ZERO;

pub trait FeltRng: RngCore {
	fn draw_element(&mut self) -> Felt;

	fn draw_word(&mut self) -> Word {
		let mut output = [ZERO; 4];
		for o in &mut output {
			*o = self.draw_element();
		}

		output
	}
}
