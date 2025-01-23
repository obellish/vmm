#[must_use]
pub fn bytes_to_words(bytes: &[u8]) -> Vec<u32> {
	let rem = bytes.len() % 4;
	let mut words = Vec::with_capacity(bytes.len() / 4 + usize::from(matches!(rem, 0)));
	let mut word = 0;

	for (i, byte) in bytes.iter().copied().enumerate() {
		word += u32::from(byte) << ((3 - (i % 4)) * 8);

		if matches!(i % 4, 3) {
			words.push(word);
			word = 0;
		}
	}

	if !matches!(rem, 0) {
		words.push(word << ((4 - rem) * 8));
	}

	words
}

#[must_use]
pub fn words_to_bytes(bytes: &[u32]) -> Vec<u8> {
	bytes.iter().flat_map(|word| word.to_be_bytes()).collect()
}
