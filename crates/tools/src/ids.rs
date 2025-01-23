use std::hash::{DefaultHasher, Hash, Hasher};

#[must_use]
pub fn gen_aux_id(name: &'static str) -> u64 {
	let mut hasher = DefaultHasher::new();
	name.hash(&mut hasher);
	hasher.finish()
}

#[cfg(test)]
mod tests {
	use super::gen_aux_id;

	#[test]
	fn works_correctly() {
		assert_eq!(gen_aux_id("test"), gen_aux_id("test"));
		assert_eq!(gen_aux_id("\0"), gen_aux_id("\0"));

		assert_ne!(gen_aux_id("test"), gen_aux_id("test2"));
		assert_ne!(gen_aux_id("test"), gen_aux_id("2test"));
	}
}
