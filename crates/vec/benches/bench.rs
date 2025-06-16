use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use vmm_vec::{SmallVec, smallvec};

const VEC_SIZE: usize = 16;
const SPILLED_SIZE: usize = 100;

trait Vector<T>: for<'a> From<&'a [T]> + Extend<T> {
	fn new() -> Self;

	fn push(&mut self, value: T);

	fn pop(&mut self) -> Option<T>;

	fn remove(&mut self, p: usize) -> T;

	fn insert(&mut self, n: usize, value: T);

	fn from_elem(value: T, n: usize) -> Self;

	fn from_elems(values: &[T]) -> Self;

	fn extend_from_slice(&mut self, other: &[T]);
}

impl<T: Copy> Vector<T> for Vec<T> {
	fn new() -> Self {
		Self::with_capacity(VEC_SIZE)
	}

	fn push(&mut self, value: T) {
		self.push(value);
	}

	fn pop(&mut self) -> Option<T> {
		self.pop()
	}

	fn remove(&mut self, p: usize) -> T {
		self.remove(p)
	}

	fn insert(&mut self, n: usize, value: T) {
		self.insert(n, value);
	}

	fn from_elem(value: T, n: usize) -> Self {
		vec![value; n]
	}

	fn from_elems(values: &[T]) -> Self {
		values.to_owned()
	}

	fn extend_from_slice(&mut self, other: &[T]) {
		Self::extend_from_slice(self, other);
	}
}

impl<T: Copy> Vector<T> for SmallVec<T, VEC_SIZE> {
	fn new() -> Self {
		Self::new()
	}

	fn push(&mut self, value: T) {
		self.push(value);
	}

	fn pop(&mut self) -> Option<T> {
		self.pop()
	}

	fn remove(&mut self, p: usize) -> T {
		self.remove(p)
	}

	fn insert(&mut self, n: usize, value: T) {
		self.insert(n, value);
	}

	fn from_elem(value: T, n: usize) -> Self {
		smallvec![value; n]
	}

	fn from_elems(values: &[T]) -> Self {
		Self::from_slice(values)
	}

	fn extend_from_slice(&mut self, other: &[T]) {
		Self::extend_from_slice(self, other);
	}
}

fn push_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	#[inline(never)]
	fn push_noinline<V>(vec: &mut V, x: u64)
	where
		V: Vector<u64>,
	{
		vec.push(x);
	}

	b.iter(|| {
		let mut vec = V::new();
		for x in 0..n {
			push_noinline(&mut vec, x);
		}
	});
}

fn insert_push_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	#[inline(never)]
	fn insert_push_noinline<V>(vec: &mut V, x: u64)
	where
		V: Vector<u64>,
	{
		vec.insert(x as usize, x);
	}

	b.iter(|| {
		let mut vec = V::new();
		for x in 0..n {
			insert_push_noinline(&mut vec, x);
		}
		vec
	});
}

fn insert_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	#[inline(never)]
	fn insert_noinline<V>(vec: &mut V, p: usize, x: u64)
	where
		V: Vector<u64>,
	{
		vec.insert(p, x);
	}

	b.iter(|| {
		let mut vec = V::new();

		vec.push(0);
		for x in 0..n {
			insert_noinline(&mut vec, 0, x);
		}
		vec
	});
}

fn remove_impl<V>(n: usize, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	#[inline(never)]
	fn remove_noinline<V>(vec: &mut V, p: usize) -> u64
	where
		V: Vector<u64>,
	{
		vec.remove(p)
	}

	b.iter(|| {
		let mut vec = V::from_elem(0, n as _);

		for _ in 0..n {
			remove_noinline(&mut vec, 0);
		}
	});
}

fn extend_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	b.iter(|| {
		let mut vec = V::new();
		vec.extend(0..n);
		vec
	});
}

fn extend_filtered_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	b.iter(|| {
		let mut vec = V::new();
		vec.extend((0..n).filter(|i| matches!(i % 2, 0)));
		vec
	});
}

#[expect(clippy::let_and_return)]
fn from_iter_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	let v: Vec<u64> = (0..n).collect();
	b.iter(|| {
		let vec = V::from(&v);
		vec
	});
}

#[expect(clippy::let_and_return)]
fn from_slice_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	let v: Vec<u64> = (0..n).collect();
	b.iter(|| {
		let vec = V::from_elems(&v);
		vec
	});
}

fn extend_from_slice_impl<V>(n: u64, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	let v: Vec<u64> = (0..n).collect();
	b.iter(|| {
		let mut vec = V::new();
		vec.extend_from_slice(&v);
		vec
	});
}

fn pushpop_impl<V>(b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	#[inline(never)]
	fn pushpop_noinline<V>(vec: &mut V, x: u64) -> Option<u64>
	where
		V: Vector<u64>,
	{
		vec.push(x);
		vec.pop()
	}

	b.iter(|| {
		let mut vec = V::new();
		for x in 0..SPILLED_SIZE as _ {
			pushpop_noinline(&mut vec, x);
		}
		vec
	});
}

#[expect(clippy::let_and_return)]
fn from_elem_impl<V>(n: usize, b: &mut Bencher<'_>)
where
	V: Vector<u64>,
{
	b.iter(|| {
		let vec = V::from_elem(42, n);
		vec
	});
}

fn push(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(push));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| push_impl::<Vec<_>>(*i as _, b));
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		push_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn push_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(push_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| push_impl::<Vec<_>>(*i as _, b));
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		push_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn insert_push(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(insert_push));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		insert_push_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		push_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn insert_push_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(insert_push_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		insert_push_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		push_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn insert(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(insert));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		insert_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		insert_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn insert_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(insert_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| insert_impl::<Vec<_>>(*i as _, b));
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		insert_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn remove(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(remove));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		remove_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		remove_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn remove_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(remove_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		remove_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		remove_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn extend(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(extend));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		extend_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		extend_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn extend_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(extend_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		extend_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		extend_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn extend_filtered(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(extend_filtered));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		extend_filtered_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		extend_filtered_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn extend_filtered_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(extend_filtered_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		extend_filtered_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		extend_filtered_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn from_iter(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(from_iter));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		from_iter_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		from_iter_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn from_iter_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(from_iter_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		from_iter_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		from_iter_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn from_slice(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(from_slice));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		from_slice_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		from_slice_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn from_slice_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(from_slice_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		from_slice_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		from_slice_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn extend_from_slice(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(extend_from_slice));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| {
		extend_from_slice_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		extend_from_slice_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn extend_from_slice_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(extend_from_slice_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| {
		extend_from_slice_impl::<Vec<_>>(*i as _, b);
	});
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		extend_from_slice_impl::<SmallVec<_, VEC_SIZE>>(*i as _, b);
	});
}

fn from_elem(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(from_elem));

	c.bench_with_input("Vec", &SPILLED_SIZE, |b, i| from_elem_impl::<Vec<_>>(*i, b));
	c.bench_with_input("SmallVec", &SPILLED_SIZE, |b, i| {
		from_elem_impl::<SmallVec<_, VEC_SIZE>>(*i, b);
	});
}

fn from_elem_small(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(from_elem_small));

	c.bench_with_input("Vec", &VEC_SIZE, |b, i| from_elem_impl::<Vec<_>>(*i, b));
	c.bench_with_input("SmallVec", &VEC_SIZE, |b, i| {
		from_elem_impl::<SmallVec<_, VEC_SIZE>>(*i, b);
	});
}

fn pushpop(c: &mut Criterion) {
	let mut c = c.benchmark_group(stringify!(pushpop));

	c.bench_function("Vec", pushpop_impl::<Vec<_>>);
	c.bench_function("SmallVec", pushpop_impl::<SmallVec<_, VEC_SIZE>>);
}

fn insert_from_slice(c: &mut Criterion) {
	c.bench_function("insert_from_slice", |b| {
		let v: Vec<u64> = (0..SPILLED_SIZE as _).collect();
		b.iter(|| {
			let mut vec = SmallVec::<u64, VEC_SIZE>::new();
			vec.insert_from_slice(0, &v);
			vec.insert_from_slice(0, &v);
			vec
		});
	});
}

criterion_group!(
	benches,
	push,
	push_small,
	insert_push,
	insert_push_small,
	insert,
	insert_small,
	remove,
	remove_small,
	extend,
	extend_small,
	extend_filtered,
	extend_filtered_small,
	from_iter,
	from_iter_small,
	from_slice,
	from_slice_small,
	extend_from_slice,
	extend_from_slice_small,
	from_elem,
	from_elem_small,
	pushpop,
	insert_from_slice
);
criterion_main!(benches);
