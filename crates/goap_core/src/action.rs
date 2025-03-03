use std::hash::{Hash, Hasher};
use bevy_reflect::Reflect;
use super::Compare;
use super::Mutator;

pub struct Action {
    pub key: String,
}
