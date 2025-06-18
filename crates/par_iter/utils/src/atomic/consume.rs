pub trait AtomicConsume {
	type Value;

	fn load_consume(&self) -> Self::Value;
}
