use std::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_reflect::{GetTypeRegistration, Typed, prelude::*};

use super::{
	Behavior, Memory, Transition,
	events::{Pause, Resume, Start, Stop},
	sequence::Sequence,
};

#[repr(transparent)]
pub struct BehaviorPlugin<T: Behavior>(PhantomData<T>);

impl<T: Behavior> Default for BehaviorPlugin<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T: RegisterableBehavior> Plugin for BehaviorPlugin<T> {
	fn build(&self, app: &mut App) {
		app.register_type::<Transition<T>>()
			.register_type::<Memory<T>>()
			.register_type::<Sequence<T>>()
			.register_required_components::<T, Transition<T>>()
			.add_event::<Start<T>>()
			.add_event::<Pause<T>>()
			.add_event::<Resume<T>>()
			.add_event::<Stop<T>>();
	}
}

pub trait RegisterableBehavior: Behavior + FromReflect + GetTypeRegistration + Typed {}

impl<T> RegisterableBehavior for T where T: Behavior + FromReflect + GetTypeRegistration + Typed {}
