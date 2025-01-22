#[derive(Debug, Clone, Copy)]
pub struct RunConfig {
	pub cycles_limit: Option<u128>,
	pub halt_on_exception: bool,
	pub print_cycles: bool,
	pub print_exceptions: bool,
	pub print_finish: bool,
	pub newline_on_finish: bool,
}

impl RunConfig {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cycles_limit: None,
			halt_on_exception: false,
			print_cycles: false,
			print_exceptions: true,
			print_finish: true,
			newline_on_finish: false,
		}
	}

	#[must_use]
	pub const fn halt_on_exception() -> Self {
		Self::new().with_halt_on_exception(true)
	}

	#[must_use]
	pub const fn verbose() -> Self {
		Self::new().be_verbose()
	}

	#[must_use]
	pub const fn quiet() -> Self {
		Self::new().be_quiet()
	}

	#[must_use]
	pub const fn with_cycles_limit(mut self, limit: Option<u128>) -> Self {
		self.cycles_limit = limit;
		self
	}

	#[must_use]
	pub const fn with_halt_on_exception(mut self, halt: bool) -> Self {
		self.halt_on_exception = halt;
		self
	}

	#[must_use]
	pub const fn with_print_cycles(mut self, print: bool) -> Self {
		self.print_cycles = print;
		self
	}

	#[must_use]
	pub const fn with_print_exceptions(mut self, print: bool) -> Self {
		self.print_exceptions = print;
		self
	}

	#[must_use]
	pub const fn with_print_finish(mut self, print: bool) -> Self {
		self.print_finish = print;
		self
	}

	#[must_use]
	pub const fn with_newline_on_finish(mut self, print: bool) -> Self {
		self.newline_on_finish = print;
		self
	}

	#[must_use]
	pub const fn be_verbose(self) -> Self {
		self.with_print_cycles(true)
			.with_print_exceptions(true)
			.with_print_finish(true)
	}

	#[must_use]
	pub const fn be_quiet(self) -> Self {
		self.with_print_cycles(false)
			.with_print_exceptions(false)
			.with_print_finish(false)
	}
}

impl Default for RunConfig {
	fn default() -> Self {
		Self::new()
	}
}
