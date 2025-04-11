use std::{alloc::System, hint::black_box, time::Instant};

use vmm_alloc::{AllocChain, UnsafeStalloc};

#[global_allocator]
static ALLOC: AllocChain<'static, UnsafeStalloc<1024, 8>, System> =
	unsafe { UnsafeStalloc::new().chain(&System) };

fn main() {
	let start = Instant::now();

	let mut big_strings = Vec::new();

	for i in 0..100_000_000 {
		black_box(String::from("hello!"));

		if matches!(i % 10000, 0) {
			big_strings.push("x".repeat(100_000));
		}
	}

	for s in big_strings {
		black_box(s);
	}

	println!("Elapsed: {}ms", start.elapsed().as_millis());
}
