use super::{JavaCodePoint, JavaStr};

mod private {
	use crate::{JavaCodePoint, JavaStr};

	pub trait Sealed {}

	impl Sealed for char {}
	impl Sealed for JavaCodePoint {}
	impl Sealed for &str {}
	impl Sealed for &JavaStr {}
	impl<F> Sealed for F where F: FnMut(JavaCodePoint) -> bool {}
	impl Sealed for &[char] {}
	impl Sealed for &[JavaCodePoint] {}
	impl Sealed for &char {}
	impl Sealed for &JavaCodePoint {}
	impl Sealed for &&str {}
	impl Sealed for &&JavaStr {}
}

pub unsafe trait JavaStrPattern: self::private::Sealed {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize>;
	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize>;
	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)>;
	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)>;
}

unsafe impl JavaStrPattern for char {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next()?;
		(ch == *self).then(|| ch.len_utf8())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next_back()?;
		(ch == *self).then(|| ch.len_utf8())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut encoded = [0; 4];
		let encoded = self.encode_utf8(&mut encoded).as_bytes();
		find(haystack.as_bytes(), encoded).map(|index| (index, encoded.len()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut encoded = [0; 4];
		let encoded = self.encode_utf8(&mut encoded).as_bytes();
		rfind(haystack.as_bytes(), encoded).map(|index| (index, encoded.len()))
	}
}

unsafe impl JavaStrPattern for JavaCodePoint {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next()?;
		(ch == *self).then(|| ch.len_utf8())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next_back()?;
		(ch == *self).then(|| ch.len_utf8())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut encoded = [0; 4];
		let encoded = self.encode_semi_utf8(&mut encoded);
		find(haystack.as_bytes(), encoded).map(|index| (index, encoded.len()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut encoded = [0; 4];
		let encoded = self.encode_semi_utf8(&mut encoded);
		rfind(haystack.as_bytes(), encoded).map(|index| (index, encoded.len()))
	}
}

unsafe impl JavaStrPattern for &str {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		haystack
			.as_bytes()
			.starts_with(self.as_bytes())
			.then_some(self.len())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		haystack
			.as_bytes()
			.ends_with(self.as_bytes())
			.then_some(self.len())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		find(haystack.as_bytes(), self.as_bytes()).map(|index| (index, self.len()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		rfind(haystack.as_bytes(), self.as_bytes()).map(|index| (index, self.len()))
	}
}

unsafe impl JavaStrPattern for &JavaStr {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		haystack
			.as_bytes()
			.starts_with(self.as_bytes())
			.then(|| self.len())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		haystack
			.as_bytes()
			.ends_with(self.as_bytes())
			.then(|| self.len())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		find(haystack.as_bytes(), self.as_bytes()).map(|index| (index, self.len()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		rfind(haystack.as_bytes(), self.as_bytes()).map(|index| (index, self.len()))
	}
}

unsafe impl<F> JavaStrPattern for F
where
	F: FnMut(JavaCodePoint) -> bool,
{
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next()?;
		self(ch).then(|| ch.len_utf8())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next_back()?;
		self(ch).then(|| ch.len_utf8())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		haystack
			.char_indices()
			.find(|(.., ch)| self(*ch))
			.map(|(index, ch)| (index, ch.len_utf8()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		haystack
			.char_indices()
			.rfind(|(.., ch)| self(*ch))
			.map(|(index, ch)| (index, ch.len_utf8()))
	}
}

unsafe impl JavaStrPattern for &[char] {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next()?;
		self.iter().any(|c| ch == *c).then(|| ch.len_utf8())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next_back()?;
		self.iter().any(|c| ch == *c).then(|| ch.len_utf8())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		haystack
			.char_indices()
			.find(|(.., ch)| self.iter().any(|c| *ch == *c))
			.map(|(index, ch)| (index, ch.len_utf8()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		haystack
			.char_indices()
			.rfind(|(.., ch)| self.iter().any(|c| *ch == *c))
			.map(|(index, ch)| (index, ch.len_utf8()))
	}
}

unsafe impl JavaStrPattern for &[JavaCodePoint] {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next()?;
		self.contains(&ch).then(|| ch.len_utf8())
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let ch = haystack.chars().next_back()?;
		self.contains(&ch).then(|| ch.len_utf8())
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		haystack
			.char_indices()
			.find(|(.., ch)| self.contains(ch))
			.map(|(index, ch)| (index, ch.len_utf8()))
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		haystack
			.char_indices()
			.rfind(|(.., ch)| self.contains(ch))
			.map(|(index, ch)| (index, ch.len_utf8()))
	}
}

unsafe impl JavaStrPattern for &char {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.prefix_len_in(haystack)
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.suffix_len_in(haystack)
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.find_in(haystack)
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.rfind_in(haystack)
	}
}

unsafe impl JavaStrPattern for &JavaCodePoint {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.prefix_len_in(haystack)
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.suffix_len_in(haystack)
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.find_in(haystack)
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.rfind_in(haystack)
	}
}

unsafe impl JavaStrPattern for &&str {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.prefix_len_in(haystack)
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.suffix_len_in(haystack)
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.find_in(haystack)
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.rfind_in(haystack)
	}
}

unsafe impl JavaStrPattern for &&JavaStr {
	fn prefix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.prefix_len_in(haystack)
	}

	fn suffix_len_in(&mut self, haystack: &JavaStr) -> Option<usize> {
		let mut ch = **self;
		ch.suffix_len_in(haystack)
	}

	fn find_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.find_in(haystack)
	}

	fn rfind_in(&mut self, haystack: &JavaStr) -> Option<(usize, usize)> {
		let mut ch = **self;
		ch.rfind_in(haystack)
	}
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
	if needle.is_empty() {
		return Some(0);
	}

	haystack
		.windows(needle.len())
		.position(|window| window == needle)
}

fn rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
	if needle.is_empty() {
		return Some(haystack.len());
	}

	haystack
		.windows(needle.len())
		.rposition(|window| window == needle)
}
