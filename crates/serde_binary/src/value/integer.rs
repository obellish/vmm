#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Integer {
	Unsigned(u128),
	Signed(i128),
}
