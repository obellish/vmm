const MDS_FREQ_BLOCK_ONE: [i64; 3] = [16, 8, 16];
const MDS_FREQ_BLOCK_TWO: [(i64, i64); 3] = [(-1, 2), (-1, 1), (4, 8)];
const MDS_FREQ_BLOCK_THREE: [i64; 3] = [-8, 1, 1];

pub const fn mds_multiply_freq(state: [u64; 12]) -> [u64; 12] {
	let [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11] = state;

	let (u0, u1, u2) = fft4_real([s0, s3, s6, s9]);
	let (u4, u5, u6) = fft4_real([s1, s4, s7, s10]);
	let (u8, u9, u10) = fft4_real([s2, s5, s8, s11]);

	let [v0, v4, v8] = block1([u0, u4, u8], MDS_FREQ_BLOCK_ONE);
	let [v1, v5, v9] = block2([u1, u5, u9], MDS_FREQ_BLOCK_TWO);
	let [v2, v6, v10] = block3([u2, u6, u10], MDS_FREQ_BLOCK_THREE);

	let [s0, s3, s6, s9] = ifft4_real((v0, v1, v2));
	let [s1, s4, s7, s10] = ifft4_real((v4, v5, v6));
	let [s2, s5, s8, s11] = ifft4_real((v8, v9, v10));

	[s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11]
}

const fn fft2_real(x: [u64; 2]) -> [i64; 2] {
	[(x[0] as i64 + x[1] as i64), (x[0] as i64 - x[1] as i64)]
}

#[inline]
const fn ifft2_real(y: [i64; 2]) -> [u64; 2] {
	[(y[0] + y[1]) as u64, (y[0] - y[1]) as u64]
}

#[inline]
const fn fft4_real(x: [u64; 4]) -> (i64, (i64, i64), i64) {
	let [z0, z2] = fft2_real([x[0], x[2]]);
	let [z1, z3] = fft2_real([x[1], x[3]]);
	let y0 = z0 + z1;
	let y1 = (z2, -z3);
	let y2 = z0 - z1;
	(y0, y1, y2)
}

#[inline]
const fn ifft4_real(y: (i64, (i64, i64), i64)) -> [u64; 4] {
	let z0 = y.0 + y.2;
	let z1 = y.0 - y.2;
	let z2 = y.1.0;
	let z3 = -y.1.1;

	let [x0, x2] = ifft2_real([z0, z2]);
	let [x1, x3] = ifft2_real([z1, z3]);

	[x0, x1, x2, x3]
}

#[inline]
const fn block1(x: [i64; 3], y: [i64; 3]) -> [i64; 3] {
	let [x0, x1, x2] = x;
	let [y0, y1, y2] = y;
	let z0 = x0 * y0 + x1 * y2 + x2 * y1;
	let z1 = x0 * y1 + x1 * y0 + x2 * y2;
	let z2 = x0 * y2 + x1 * y1 + x2 * y0;

	[z0, z1, z2]
}

#[inline]
const fn block2(x: [(i64, i64); 3], y: [(i64, i64); 3]) -> [(i64, i64); 3] {
	let [(x0r, x0i), (x1r, x1i), (x2r, x2i)] = x;
	let [(y0r, y0i), (y1r, y1i), (y2r, y2i)] = y;
	let x0s = x0r + x0i;
	let x1s = x1r + x1i;
	let x2s = x2r + x2i;
	let y0s = y0r + y0i;
	let y1s = y1r + y1i;
	let y2s = y2r + y2i;

	let m0 = (x0r * y0r, x0i * y0i);
	let m1 = (x1r * y2r, x1i * y2i);
	let m2 = (x2r * y1r, x2i * y1i);
	let z0r = (m0.0 - m0.1) + (x1s * y2s - m1.0 - m1.1) + (x2s * y1s - m2.0 - m2.1);
	let z0i = (x0s * y0s - m0.0 - m0.1) + (-m1.0 + m1.1) + (-m2.0 + m2.1);
	let z0 = (z0r, z0i);

	let m0 = (x0r * y1r, x0i * y1i);
	let m1 = (x1r * y0r, x1i * y0i);
	let m2 = (x2r * y2r, x2i * y2i);
	let z1r = (m0.0 - m0.1) + (m1.0 - m1.1) + (x2s * y2s - m2.0 - m2.1);
	let z1i = (x0s * y1s - m0.0 - m0.1) + (x1s * y0s - m1.0 - m1.1) + (-m2.0 + m2.1);
	let z1 = (z1r, z1i);

	let m0 = (x0r * y2r, x0i * y2i);
	let m1 = (x1r * y1r, x1i * y1i);
	let m2 = (x2r * y0r, x2i * y0i);
	let z2r = (m0.0 - m0.1) + (m1.0 - m1.1) + (m2.0 - m2.1);
	let z2i = (x0s * y2s - m0.0 - m0.1) + (x1s * y1s - m1.0 - m1.1) + (x2s * y0s - m2.0 - m2.1);
	let z2 = (z2r, z2i);

	[z0, z1, z2]
}

#[inline]
const fn block3(x: [i64; 3], y: [i64; 3]) -> [i64; 3] {
	let [x0, x1, x2] = x;
	let [y0, y1, y2] = y;
	let z0 = x0 * y0 - x1 * y2 - x2 * y1;
	let z1 = x0 * y1 + x1 * y0 - x2 * y2;
	let z2 = x0 * y2 + x1 * y1 + x2 * y0;

	[z0, z1, z2]
}

#[cfg(test)]
mod tests {
	
}
