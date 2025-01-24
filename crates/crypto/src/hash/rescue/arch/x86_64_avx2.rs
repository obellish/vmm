use core::arch::x86_64::{
	__m256i, _mm256_add_epi64, _mm256_and_si256, _mm256_blend_epi32, _mm256_castps_si256,
	_mm256_castsi256_pd, _mm256_castsi256_ps, _mm256_cmpgt_epi32, _mm256_loadu_si256,
	_mm256_movehdup_ps, _mm256_moveldup_ps, _mm256_mul_epu32, _mm256_set1_epi64x,
	_mm256_slli_epi64, _mm256_srli_epi64, _mm256_storeu_si256, _mm256_sub_epi64, _mm256_testz_pd,
	_mm256_xor_si256,
};

#[inline]
pub fn branch_hint() {
	#[cfg(any(
		target_arch = "aarch64",
		target_arch = "arm",
		target_arch = "riscv32",
		target_arch = "riscv64",
		target_arch = "x86",
		target_arch = "x86_64"
	))]
	unsafe {
		core::arch::asm!("", options(nomem, nostack, preserves_flags));
	}
}

macro_rules! map3 {
	($f:ident:: < $l:literal > , $v:ident) => {
		unsafe { ($f::<$l>($v.0), $f::<$l>($v.1), $f::<$l>($v.2)) }
	};
	($f:ident:: < $l:literal > , $v1:ident, $v2:ident) => {
		unsafe {
			(
				$f::<$l>($v1.0, $v2.0),
				$f::<$l>($v1.1, $v2.1),
				$f::<$l>($v1.2, $v2.2),
			)
		}
	};
	($f:ident, $v:ident) => {
		unsafe { ($f($v.0), $f($v.1), $f($v.2)) }
	};
	($f:ident, $v0:ident, $v1:ident) => {
		unsafe { ($f($v0.0, $v1.0), $f($v0.1, $v1.1), $f($v0.2, $v1.2)) }
	};
	($f:ident,rep $v0:ident, $v1:ident) => {
		($f($v0, $v1.0), $f($v0, $v1.1), $f($v0, $v1.2))
	};
	($f:ident, $v0:ident,rep $v1:ident) => {
		unsafe { ($f($v0.0, $v1), $f($v0.1, $v1), $f($v0.2, $v1)) }
	};
}

#[inline]
unsafe fn square3(
	x: (__m256i, __m256i, __m256i),
) -> ((__m256i, __m256i, __m256i), (__m256i, __m256i, __m256i)) {
	let x_hi = {
		let x_ps = map3!(_mm256_castsi256_ps, x);
		let x_hi_ps = map3!(_mm256_movehdup_ps, x_ps);
		map3!(_mm256_castps_si256, x_hi_ps)
	};

	let mul_ll = map3!(_mm256_mul_epu32, x, x);
	let mul_lh = map3!(_mm256_mul_epu32, x, x_hi);
	let mul_hh = map3!(_mm256_mul_epu32, x_hi, x_hi);

	let mul_ll_hi = map3!(_mm256_srli_epi64::<33>, mul_ll);
	let t0 = map3!(_mm256_add_epi64, mul_lh, mul_ll_hi);
	let t0_hi = map3!(_mm256_srli_epi64::<31>, t0);
	let res_hi = map3!(_mm256_add_epi64, mul_hh, t0_hi);

	let mul_lh_lo = map3!(_mm256_slli_epi64::<33>, mul_lh);
	let res_lo = map3!(_mm256_add_epi64, mul_ll, mul_lh_lo);

	(res_lo, res_hi)
}

#[inline]
unsafe fn mul3(
	x: (__m256i, __m256i, __m256i),
	y: (__m256i, __m256i, __m256i),
) -> ((__m256i, __m256i, __m256i), (__m256i, __m256i, __m256i)) {
	let epsilon = unsafe { _mm256_set1_epi64x(0xffff_ffff) };
	let x_hi = {
		let x_ps = map3!(_mm256_castsi256_ps, x);
		let x_hi_ps = map3!(_mm256_movehdup_ps, x_ps);
		map3!(_mm256_castps_si256, x_hi_ps)
	};
	let y_hi = {
		let y_ps = map3!(_mm256_castsi256_ps, y);
		let y_hi_ps = map3!(_mm256_movehdup_ps, y_ps);
		map3!(_mm256_castps_si256, y_hi_ps)
	};

	let mul_ll = map3!(_mm256_mul_epu32, x, y);
	let mul_lh = map3!(_mm256_mul_epu32, x, y_hi);
	let mul_hl = map3!(_mm256_mul_epu32, x_hi, y);
	let mul_hh = map3!(_mm256_mul_epu32, x_hi, y_hi);

	let mul_ll_hi = map3!(_mm256_srli_epi64::<32>, mul_ll);
	let t0 = map3!(_mm256_add_epi64, mul_hl, mul_ll_hi);

	let t0_lo = map3!(_mm256_and_si256, t0, rep epsilon);
	let t0_hi = map3!(_mm256_srli_epi64::<32>, t0);
	let t1 = map3!(_mm256_add_epi64, mul_lh, t0_lo);
	let t2 = map3!(_mm256_add_epi64, mul_hh, t0_hi);

	let t1_hi = map3!(_mm256_srli_epi64::<32>, t1);
	let res_hi = map3!(_mm256_add_epi64, t2, t1_hi);

	let t1_lo = {
		let t1_ps = map3!(_mm256_castsi256_ps, t1);
		let t1_lo_ps = map3!(_mm256_moveldup_ps, t1_ps);
		map3!(_mm256_castps_si256, t1_lo_ps)
	};
	let res_lo = map3!(_mm256_blend_epi32::<0xaa>, mul_ll, t1_lo);

	(res_lo, res_hi)
}

#[inline]
unsafe fn add_small(
	x_s: (__m256i, __m256i, __m256i),
	y: (__m256i, __m256i, __m256i),
) -> (__m256i, __m256i, __m256i) {
	let res_wrapped_s = map3!(_mm256_add_epi64, x_s, y);
	let mask = map3!(_mm256_cmpgt_epi32, x_s, res_wrapped_s);
	let wrapback_amt = map3!(_mm256_srli_epi64::<32>, mask);

	map3!(_mm256_add_epi64, res_wrapped_s, wrapback_amt)
}

#[inline]
unsafe fn maybe_adj_sub(res_wrapped_s: __m256i, mask: __m256i) -> __m256i {
	let mask_pd = unsafe { _mm256_castsi256_pd(mask) };

	if unsafe { matches!(_mm256_testz_pd(mask_pd, mask_pd), 1) } {
		res_wrapped_s
	} else {
		branch_hint();

		let adj_amount = unsafe { _mm256_srli_epi64::<32>(mask) };
		unsafe { _mm256_sub_epi64(res_wrapped_s, adj_amount) }
	}
}

#[inline]
unsafe fn sub_tiny(
	x_s: (__m256i, __m256i, __m256i),
	y: (__m256i, __m256i, __m256i),
) -> (__m256i, __m256i, __m256i) {
	let res_wrapped_s = map3!(_mm256_sub_epi64, x_s, y);
	let mask = map3!(_mm256_cmpgt_epi32, res_wrapped_s, x_s);
	map3!(maybe_adj_sub, res_wrapped_s, mask)
}

#[inline]
unsafe fn reduce3(
	(lo0, hi0): ((__m256i, __m256i, __m256i), (__m256i, __m256i, __m256i)),
) -> (__m256i, __m256i, __m256i) {
	let sign_bit = unsafe { _mm256_set1_epi64x(i64::MIN) };
	let epsilon = unsafe { _mm256_set1_epi64x(0xffff_ffff) };
	let lo0_s = map3!(_mm256_xor_si256, lo0, rep sign_bit);
	let hi_hi0 = map3!(_mm256_srli_epi64::<32>, hi0);
	let lo1_s = unsafe { sub_tiny(lo0_s, hi_hi0) };
	let t1 = map3!(_mm256_mul_epu32, hi0, rep epsilon);
	let lo2_s = unsafe { add_small(lo1_s, t1) };
	map3!(_mm256_xor_si256, lo2_s, rep sign_bit)
}

#[inline]
unsafe fn mul_reduce(
	a: (__m256i, __m256i, __m256i),
	b: (__m256i, __m256i, __m256i),
) -> (__m256i, __m256i, __m256i) {
	unsafe { reduce3(mul3(a, b)) }
}

#[inline]
unsafe fn square_reduce(state: (__m256i, __m256i, __m256i)) -> (__m256i, __m256i, __m256i) {
	unsafe { reduce3(square3(state)) }
}

#[inline]
unsafe fn exp_acc(
	high: (__m256i, __m256i, __m256i),
	low: (__m256i, __m256i, __m256i),
	exp: usize,
) -> (__m256i, __m256i, __m256i) {
	let mut result = high;
	for _ in 0..exp {
		result = unsafe { square_reduce(result) };
	}
	unsafe { mul_reduce(result, low) }
}

#[inline]
unsafe fn do_apply_sbox(state: (__m256i, __m256i, __m256i)) -> (__m256i, __m256i, __m256i) {
	let state2 = unsafe { square_reduce(state) };
	let state4_unreduced = unsafe { square3(state2) };
	let state3_unreduced = unsafe { mul3(state2, state) };
	let state4 = unsafe { reduce3(state4_unreduced) };
	let state3 = unsafe { reduce3(state3_unreduced) };
	let state7_unreduced = unsafe { mul3(state3, state4) };
	unsafe { reduce3(state7_unreduced) }
}

#[inline]
unsafe fn do_apply_inv_sbox(state: (__m256i, __m256i, __m256i)) -> (__m256i, __m256i, __m256i) {
	let t1 = unsafe { square_reduce(state) };
	let t2 = unsafe { square_reduce(t1) };
	let t3 = unsafe { exp_acc(t2, t2, 3) };
	let t4 = unsafe { exp_acc(t3, t3, 6) };
	let t5 = unsafe { exp_acc(t4, t4, 12) };
	let t6 = unsafe { exp_acc(t5, t3, 6) };
	let t7 = unsafe { exp_acc(t6, t6, 31) };

	let a = unsafe { square_reduce(square_reduce(mul_reduce(square_reduce(t7), t6))) };
	let b = unsafe { mul_reduce(t1, mul_reduce(t2, state)) };

	unsafe { mul_reduce(a, b) }
}

#[allow(clippy::cast_ptr_alignment)]
#[inline]
unsafe fn avx2_load(state: &[u64; 12]) -> (__m256i, __m256i, __m256i) {
	unsafe {
		(
			_mm256_loadu_si256(state[0..4].as_ptr().cast::<__m256i>()),
			_mm256_loadu_si256(state[4..8].as_ptr().cast::<__m256i>()),
			_mm256_loadu_si256(state[8..12].as_ptr().cast::<__m256i>()),
		)
	}
}

#[allow(clippy::cast_ptr_alignment)]
#[inline]
unsafe fn avx2_store(buf: &mut [u64; 12], state: (__m256i, __m256i, __m256i)) {
	unsafe { _mm256_storeu_si256(buf[0..4].as_mut_ptr().cast::<__m256i>(), state.0) };
	unsafe { _mm256_storeu_si256(buf[4..8].as_mut_ptr().cast::<__m256i>(), state.1) };
	unsafe { _mm256_storeu_si256(buf[8..12].as_mut_ptr().cast::<__m256i>(), state.2) };
}

#[inline]
pub unsafe fn apply_sbox(buffer: &mut [u64; 12]) {
	let mut state = unsafe { avx2_load(buffer) };
	state = unsafe { do_apply_sbox(state) };
	unsafe { avx2_store(buffer, state) };
}

pub unsafe fn apply_inv_sbox(buffer: &mut [u64; 12]) {
	let mut state = unsafe { avx2_load(buffer) };
	state = unsafe { do_apply_inv_sbox(state) };
	unsafe { avx2_store(buffer, state) };
}
