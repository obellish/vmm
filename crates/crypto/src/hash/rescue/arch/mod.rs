#[cfg(target_feature = "sve")]
pub mod optimized {

	use crate::{Felt, hash::rescue::STATE_WIDTH};

	mod ffi {
		use core::ffi::c_ulong;

		#[link(name = "rpo_sve", kind = "static")]
		unsafe extern "C" {
			pub fn add_constants_and_apply_sbox(
				state: *mut c_ulong,
				constants: *const c_ulong,
			) -> bool;

			pub fn add_constants_and_apply_inv_sbox(
				state: *mut c_ulong,
				constants: *const c_ulong,
			) -> bool;
		}
	}

	pub fn add_constants_and_apply_sbox(
		state: &mut [Felt; STATE_WIDTH],
		ark: &[Felt; STATE_WIDTH],
	) -> bool {
		unsafe { ffi::add_constants_and_apply_sbox(state.as_mut_ptr().cast(), ark.as_ptr().cast()) }
	}

	pub fn add_constants_and_apply_inv_sbox(
		state: &mut [Felt; STATE_WIDTH],
		ark: &[Felt; STATE_WIDTH],
	) -> bool {
		unsafe {
			ffi::add_constants_and_apply_inv_sbox(state.as_mut_ptr().cast(), ark.as_ptr().cast())
		}
	}
}

#[cfg(target_feature = "avx2")]
mod x86_64_avx2;

#[allow(clippy::trivially_copy_pass_by_ref)]
#[cfg(target_feature = "avx2")]
pub mod optimized {
	use super::x86_64_avx2::{apply_inv_sbox, apply_sbox};
	use crate::{
		Felt,
		hash::rescue::{STATE_WIDTH, add_constants},
	};

	#[inline]
	pub fn add_constants_and_apply_sbox(
		state: &mut [Felt; STATE_WIDTH],
		ark: &[Felt; STATE_WIDTH],
	) -> bool {
		add_constants(state, ark);
		unsafe {
			apply_sbox(
				&mut *core::ptr::from_mut::<[Felt; STATE_WIDTH]>(state)
					.cast::<[u64; STATE_WIDTH]>(),
			);
		}

		true
	}

	#[inline]
	pub fn add_constants_and_apply_inv_sbox(
		state: &mut [Felt; STATE_WIDTH],
		ark: &[Felt; STATE_WIDTH],
	) -> bool {
		add_constants(state, ark);
		unsafe {
			apply_inv_sbox(&mut *core::ptr::from_mut::<[Felt; 12]>(state).cast::<[u64; 12]>());
		}

		true
	}
}

#[cfg(not(any(target_feature = "avx2", target_feature = "sve")))]
pub mod optimized {
	use crate::{Felt, hash::rescue::STATE_WIDTH};

	#[inline]
	pub fn add_constants_and_apply_sbox(
		_state: &mut [Felt; STATE_WIDTH],
		_ark: &[Felt; STATE_WIDTH],
	) -> bool {
		false
	}

	#[inline]
	pub fn add_constants_and_apply_inv_sbox(
		_state: &mut [Felt; STATE_WIDTH],
		_ark: &[Felt; STATE_WIDTH],
	) -> bool {
		false
	}
}
