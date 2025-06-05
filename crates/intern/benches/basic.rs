use std::mem;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use quickcheck::{Arbitrary, Gen};
use vmm_intern::{HashInterner, OrdInterner};

fn large(size: usize) -> Vec<u8> {
	Arbitrary::arbitrary(&mut Gen::new(size))
}

fn hash(c: &mut Criterion) {
	let mut c = c.benchmark_group("hash");

	c.bench_function("intern fresh", |b| {
		b.iter_batched_ref(
			HashInterner::<str>::new,
			|i| i.intern_ref("hello"),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern known", |b| {
		b.iter_batched_ref(
			|| {
				let i = HashInterner::<str>::new();
				i.intern_ref("hello");
				i
			},
			|i| i.intern_ref("hello"),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern & drop fresh", |b| {
		b.iter_batched_ref(
			HashInterner::<str>::new,
			|i| {
				i.intern_ref("hello");
			},
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern & drop known", |b| {
		b.iter_batched_ref(
			|| {
				let i = HashInterner::<str>::new();
				i.intern_ref("hello");
				i
			},
			|i| {
				i.intern_ref("hello");
			},
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern when crowded", |b| {
		b.iter_batched_ref(
			|| {
				let i = HashInterner::<u32>::new();
				for n in 0..100_000 {
					mem::forget(i.intern_sized(n));
				}
				i
			},
			|i| i.intern_sized(12_345_678),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern large values", |b| {
		b.iter_batched_ref(
			|| {
				let i = HashInterner::<[u8]>::new();
				for _ in 0..100 {
					mem::forget(i.intern_box(large(500_000).into()));
				}
				let v = i.intern_box(large(500_000).into());
				(i, v)
			},
			|(i, v)| i.intern_ref(v.as_ref()),
			BatchSize::LargeInput,
		);
	});
	c.bench_function("intern medium existing values", |b| {
		b.iter_batched_ref(
			|| {
				let i = HashInterner::<[u8]>::new();
				for _ in 0..1000 {
					mem::forget(i.intern_box(large(50_000).into()));
				}
				let v = i.intern_box(large(50_000).into());
				(i, v)
			},
			|(i, v)| i.intern_ref(v.as_ref()),
			BatchSize::LargeInput,
		);
	});

	c.finish();
}

fn ord(c: &mut Criterion) {
	let mut c = c.benchmark_group("ord");

	c.bench_function("intern fresh", |b| {
		b.iter_batched_ref(
			OrdInterner::<str>::new,
			|i| i.intern_ref("hello"),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern known", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<str>::new();
				i.intern_ref("hello");
				i
			},
			|i| i.intern_ref("hello"),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern & drop fresh", |b| {
		b.iter_batched_ref(
			OrdInterner::<str>::new,
			|i| {
				i.intern_ref("hello");
			},
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern & drop known", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<str>::new();
				i.intern_ref("hello");
				i
			},
			|i| {
				i.intern_ref("hello");
			},
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern when crowded", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<u32>::new();
				for n in 0..100_000 {
					mem::forget(i.intern_sized(n));
				}
				i
			},
			|i| i.intern_sized(12_345_678),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern when moderately crowded", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<u32>::new();
				for n in 0..100 {
					mem::forget(i.intern_sized(n));
				}
				i
			},
			|i| i.intern_sized(12_345_678),
			BatchSize::SmallInput,
		);
	});
	c.bench_function("intern large values", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<[u8]>::new();
				for _ in 0..100 {
					mem::forget(i.intern_box(large(500_000).into()));
				}
				i
			},
			|i| i.intern_box(large(500_000).into()),
			BatchSize::LargeInput,
		);
	});
	c.bench_function("intern large existing values", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<[u8]>::new();
				for _ in 0..100 {
					mem::forget(i.intern_box(large(500_000).into()));
				}
				let v = i.intern_box(large(500_000).into());
				(i, v)
			},
			|(i, v)| i.intern_ref(v.as_ref()),
			BatchSize::LargeInput,
		);
	});
	c.bench_function("intern medium existing values", |b| {
		b.iter_batched_ref(
			|| {
				let i = OrdInterner::<[u8]>::new();
				for _ in 0..1000 {
					mem::forget(i.intern_box(large(50_000).into()));
				}
				let v = i.intern_box(large(50_000).into());
				(i, v)
			},
			|(i, v)| i.intern_ref(v.as_ref()),
			BatchSize::LargeInput,
		);
	});

	c.finish();
}

criterion_group!(benches, hash, ord);
criterion_main!(benches);
