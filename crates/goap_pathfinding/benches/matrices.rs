use criterion::{Criterion, criterion_group, criterion_main};
use goap_pathfinding::matrix::Matrix;

fn transpose(c: &mut Criterion) {
	let data = (0..100 * 100).collect::<Vec<_>>();
	let mut m = Matrix::try_square_from_iter(data).unwrap();

	c.bench_function(stringify!(transpose), |b| b.iter(|| m.transpose()));
}

fn transpose_non_square(c: &mut Criterion) {
	let data = (0..100 * 100).collect::<Vec<_>>();
	let mut m = Matrix::try_from_iter(1000, 10, data).unwrap();

	c.bench_function(stringify!(transpose_non_square), |b| {
		b.iter(|| m.transpose());
	});
}

criterion_group!(benches, transpose, transpose_non_square);
criterion_main!(benches);
