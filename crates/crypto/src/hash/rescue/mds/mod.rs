mod freq;

pub use self::freq::mds_multiply_freq;
use super::STATE_WIDTH;
use crate::{Felt, ZERO};

#[inline]
pub fn apply_mds(state: &mut [Felt; STATE_WIDTH]) {
	let mut result = [ZERO; STATE_WIDTH];

	let mut state_l = [0u64; STATE_WIDTH];
	let mut state_h = [0u64; STATE_WIDTH];

	for r in 0..STATE_WIDTH {
		let s = state[r].inner();
		state_h[r] = s >> 32;
		state_l[r] = u64::from(s as u32);
	}

	let state_l = mds_multiply_freq(state_l);
	let state_h = mds_multiply_freq(state_h);

	for r in 0..STATE_WIDTH {
		let s = u128::from(state_l[r]) + (u128::from(state_h[r]) << 32);
		let s_hi = (s >> 64) as u64;
		let s_lo = s as u64;
		let z = (s_hi << 32) - s_hi;
		let (res, over) = s_lo.overflowing_add(z);

		result[r] =
			Felt::from_mont(res.wrapping_add(u64::from(0u32.wrapping_sub(u32::from(over)))));
	}
	*state = result;
}

pub const MDS: [[Felt; STATE_WIDTH]; STATE_WIDTH] = [
	[
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
	],
	[
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
	],
	[
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
	],
	[
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
	],
	[
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
	],
	[
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
	],
	[
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
	],
	[
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
	],
	[
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
	],
	[
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
		Felt::new(8),
	],
	[
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
		Felt::new(23),
	],
	[
		Felt::new(23),
		Felt::new(8),
		Felt::new(26),
		Felt::new(13),
		Felt::new(10),
		Felt::new(9),
		Felt::new(7),
		Felt::new(6),
		Felt::new(22),
		Felt::new(21),
		Felt::new(8),
		Felt::new(7),
	],
];
