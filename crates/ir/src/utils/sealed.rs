use crate::{BlockInstruction, Instruction, SuperInstruction};

pub trait Sealed {}

impl<T: Sealed> Sealed for [T] {}
impl Sealed for Instruction {}
impl Sealed for BlockInstruction {}
impl Sealed for SuperInstruction {}
