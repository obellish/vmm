#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::{
	collections::hash_map::RandomState,
	fmt::{Display, Formatter, Result as FmtResult},
	hash::{BuildHasher, Hasher},
	panic::{self, AssertUnwindSafe},
	time::{Duration, Instant},
};

#[doc(inline)]
pub use arbitrary;

pub fn run_test<P>(property: P) -> Test<P>
where
	P: FnMut(&mut self::arbitrary::Unstructured<'_>) -> self::arbitrary::Result<()>,
{
	let options = Options::default();
	Test {
		property,
		options,
		done: false,
	}
}

pub struct Test<P>
where
	P: FnMut(&mut self::arbitrary::Unstructured<'_>) -> self::arbitrary::Result<()>,
{
	property: P,
	options: Options,
	done: bool,
}

impl<P> Test<P>
where
	P: FnMut(&mut self::arbitrary::Unstructured<'_>) -> self::arbitrary::Result<()>,
{
	#[must_use]
	pub const fn size_min(mut self, size: u32) -> Self {
		self.options.size_min = size;
		self
	}

	#[must_use]
	pub const fn size_max(mut self, size: u32) -> Self {
		self.options.size_max = size;
		self
	}

	#[must_use]
	pub const fn budget(mut self, value: Duration) -> Self {
		self.options.budget = Some(value);
		self
	}

	#[must_use]
	pub const fn budget_ms(self, value: u64) -> Self {
		self.budget(Duration::from_millis(value))
	}

	#[must_use]
	pub const fn seed(mut self, seed: u64) -> Self {
		self.options.seed = Some(Seed::new(seed));
		self
	}

	#[must_use]
	pub const fn minimize(mut self) -> Self {
		self.options.minimize = true;
		self
	}

	pub fn run(&mut self) {
		self.context().run();
	}

	fn context(&mut self) -> Context<'_, '_> {
		assert!(!self.done);
		self.done = true;
		Context {
			property: &mut self.property,
			options: &self.options,
			buffer: Vec::new(),
		}
	}
}

impl<P> Drop for Test<P>
where
	P: FnMut(&mut self::arbitrary::Unstructured<'_>) -> self::arbitrary::Result<()>,
{
	fn drop(&mut self) {
		if !self.done {
			self.run();
		}
	}
}

struct Options {
	size_min: u32,
	size_max: u32,
	budget: Option<Duration>,
	seed: Option<Seed>,
	minimize: bool,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			size_min: 32,
			size_max: 65_536,
			budget: None,
			seed: None,
			minimize: false,
		}
	}
}

struct Context<'a, 'b> {
	property: DynProperty<'a>,
	options: &'b Options,
	buffer: Vec<u8>,
}

impl Context<'_, '_> {
	fn run(&mut self) {
		let budget = self
			.options
			.budget
			.or_else(env_budget)
			.unwrap_or_else(|| Duration::from_millis(100));

		match (self.options.seed.or_else(env_seed), self.options.minimize) {
			(None, false) => self.run_search(budget),
			(None, true) => panic!("can't minimize without a seed"),
			(Some(seed), false) => self.run_reproduce(seed),
			(Some(seed), true) => self.run_minimize(seed, budget),
		}
	}

	fn run_search(&mut self, budget: Duration) {
		let t = Instant::now();

		let mut last_result = Ok(());
		let mut seen_success = false;

		let mut size = self.options.size_min;
		'search: loop {
			for _ in 0..3 {
				if t.elapsed() > budget {
					break 'search;
				}

				let seed = Seed::generate(size);
				{
					let guard = PrintSeedOnPanic::new(seed);
					last_result = self.try_seed(seed);
					seen_success = seen_success || last_result.is_ok();
					guard.defuse();
				}
			}

			let bigger = u64::from(size).saturating_mul(5) / 4;
			size = bigger.clamp(0, u64::from(self.options.size_max)) as u32;
		}

		if !seen_success {
			let error = last_result.unwrap_err();
			panic!("no fitting seeds, last error: {error}");
		}
	}

	fn run_reproduce(&mut self, seed: Seed) {
		let guard = PrintSeedOnPanic::new(seed);
		self.try_seed(seed)
			.unwrap_or_else(|error| panic!("{error}"));
		guard.defuse();
	}

	fn run_minimize(&mut self, seed: Seed, budget: Duration) {
		let old_hook = panic::take_hook();
		panic::set_hook(Box::new(|_| ()));

		if !self.try_seed_panics(seed) {
			panic::set_hook(old_hook);
			panic!("seed {seed:#} did not panic");
		}

		let mut seed = seed;
		let t = Instant::now();

		let minimizers = [|s| s / 2, |s| s * 9 / 10, |s| s - 1];
		let mut minimizer = 0;

		let mut last_minimization = Instant::now();
		'search: loop {
			let size = seed.size();
			eprintln!(
				"seed {seed:#}, seed size {size}, search time {:0.2?}",
				t.elapsed()
			);

			if matches!(size, 0) {
				break;
			}

			loop {
				if t.elapsed() > budget {
					break 'search;
				}

				if last_minimization.elapsed() > budget / 5 && minimizer < minimizers.len() - 1 {
					minimizer += 1;
				}

				let size = minimizers[minimizer](size);
				let candidate_seed = Seed::generate(size);
				if self.try_seed_panics(candidate_seed) {
					seed = candidate_seed;
					last_minimization = Instant::now();
					continue 'search;
				}
			}
		}

		panic::set_hook(old_hook);
		let size = seed.size();
		eprintln!("minimized");
		eprintln!(
			"seed {seed:#}, seed size {size}, search time {:0.2?}",
			t.elapsed()
		);
		panic!("minimization failed successfully");
	}

	fn try_seed(&mut self, seed: Seed) -> self::arbitrary::Result<()> {
		seed.fill(&mut self.buffer);
		let mut u = self::arbitrary::Unstructured::new(&self.buffer);
		(self.property)(&mut u)
	}

	fn try_seed_panics(&mut self, seed: Seed) -> bool {
		let mut me = AssertUnwindSafe(self);
		panic::catch_unwind(move || {
			let _ = me.try_seed(seed);
		})
		.is_err()
	}
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Seed {
	repr: u64,
}

impl Seed {
	const fn new(repr: u64) -> Self {
		Self { repr }
	}

	fn generate(size: u32) -> Self {
		let raw = RandomState::new().build_hasher().finish();
		let repr = u64::from(size) | (raw << u32::BITS);
		Self::new(repr)
	}

	const fn size(self) -> u32 {
		self.repr as u32
	}

	const fn rand(self) -> u32 {
		(self.repr >> u32::BITS) as u32
	}

	fn fill(self, buf: &mut Vec<u8>) {
		buf.clear();
		buf.reserve(self.size() as usize);
		let mut random = self.rand();
		let mut rng = std::iter::repeat_with(move || {
			random ^= random << 13;
			random ^= random >> 17;
			random ^= random << 5;
			random
		});
		while buf.len() < self.size() as usize {
			buf.extend(rng.next().unwrap().to_le_bytes());
		}
	}
}

impl Display for Seed {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if f.alternate() {
			write!(f, "\x1b[1m0x{:016x}\x1b[0m", self.repr)
		} else {
			Display::fmt(&self.repr, f)
		}
	}
}

struct PrintSeedOnPanic {
	seed: Seed,
	active: bool,
}

impl PrintSeedOnPanic {
	const fn new(seed: Seed) -> Self {
		Self { seed, active: true }
	}

	fn defuse(mut self) {
		self.active = false;
	}
}

impl Drop for PrintSeedOnPanic {
	fn drop(&mut self) {
		if self.active {
			eprintln!("\nvmm_test failed!\n    Seed: {:#}\n\n", self.seed);
		}
	}
}

type DynProperty<'a> =
	&'a mut dyn FnMut(&mut self::arbitrary::Unstructured<'_>) -> self::arbitrary::Result<()>;

fn env_budget() -> Option<Duration> {
	let var = std::env::var("TEST_BUDGET_MS").ok()?;
	let ms = var.parse::<u64>().ok()?;
	Some(Duration::from_millis(ms))
}

fn env_seed() -> Option<Seed> {
	let var = std::env::var("TEST_SEED").ok()?;
	let repr = u64::from_str_radix(
		if let Some(stripped_var) = var.strip_prefix("0x") {
			stripped_var
		} else {
			&var
		},
		16,
	)
	.ok()?;

	Some(Seed::new(repr))
}
