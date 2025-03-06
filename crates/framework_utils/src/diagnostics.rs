use bevy_ecs::prelude::*;

pub trait RunSystemLoop: Sized {
	fn run_system_loop<Out, Marker>(
		self,
		n: usize,
		system: impl IntoSystem<(), Out, Marker>,
	) -> Vec<Out> {
		self.run_system_loop_with(n, || (), system)
	}

	fn run_system_loop_with<InputSource, I: SystemInput, Out, Marker>(
		self,
		n: usize,
		input: InputSource,
		system: impl IntoSystem<I, Out, Marker>,
	) -> Vec<Out>
	where
		InputSource: FnMut() -> I::Inner<'static>;
}

impl RunSystemLoop for &mut World {
	fn run_system_loop_with<InputSource, I: SystemInput, Out, Marker>(
		self,
		n: usize,
		mut input: InputSource,
		system: impl IntoSystem<I, Out, Marker>,
	) -> Vec<Out>
	where
		InputSource: FnMut() -> I::Inner<'static>,
	{
		let mut system = IntoSystem::into_system(system);
		system.initialize(self);
		let mut outs = Vec::new();
		for _ in 0..n {
			let out = system.run(input(), self);
			outs.push(out);
			system.apply_deferred(self);
		}

		outs
	}
}
