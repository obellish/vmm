mod document;
mod print;

use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};

pub use self::document::{Document, concat, const_text, display, flatten, indent, nl, split, text};

struct Prettier<'a, P: ?Sized>(&'a P);

impl<P> Display for Prettier<'_, P>
where
	P: ?Sized + PrettyPrint,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.0.pretty_print(f)
	}
}

pub trait PrettyPrint {
	fn render(&self) -> Document;

	fn to_pretty_string(&self) -> String {
		format!("{:width$}", Prettier(self), width = 80)
	}

	fn pretty_print(&self, f: &mut Formatter<'_>) -> FmtResult {
		let doc = self.render();
		let width = f.width().unwrap_or(80);
		self::print::pretty_print(&doc, width, f)
	}
}

impl<T> PrettyPrint for &T
where
	T: ?Sized + PrettyPrint,
{
	fn render(&self) -> Document {
		(**self).render()
	}

	fn to_pretty_string(&self) -> String {
		(**self).to_pretty_string()
	}

	fn pretty_print(&self, f: &mut Formatter<'_>) -> FmtResult {
		(**self).pretty_print(f)
	}
}

impl PrettyPrint for str {
	fn render(&self) -> Document {
		self.lines()
			.map(text)
			.reduce(|acc, doc| match acc {
				Document::Empty => doc + nl(),
				other => other + doc + nl(),
			})
			.unwrap_or_default()
	}
}

impl PrettyPrint for String {
	fn render(&self) -> Document {
		self.as_str().render()
	}
}

impl PrettyPrint for alloc::borrow::Cow<'_, str> {
	fn render(&self) -> Document {
		PrettyPrint::render(self.as_ref())
	}

	fn pretty_print(&self, f: &mut Formatter<'_>) -> FmtResult {
		PrettyPrint::pretty_print(self.as_ref(), f)
	}
}

impl<T: PrettyPrint> PrettyPrint for alloc::boxed::Box<T> {
	fn render(&self) -> Document {
		PrettyPrint::render(self.as_ref())
	}

	fn pretty_print(&self, f: &mut Formatter<'_>) -> FmtResult {
		PrettyPrint::pretty_print(self.as_ref(), f)
	}
}

impl<T: PrettyPrint> PrettyPrint for alloc::rc::Rc<T> {
	fn render(&self) -> Document {
		PrettyPrint::render(self.as_ref())
	}

	fn pretty_print(&self, f: &mut Formatter<'_>) -> FmtResult {
		PrettyPrint::pretty_print(self.as_ref(), f)
	}
}

impl<T: PrettyPrint> PrettyPrint for alloc::sync::Arc<T> {
	fn render(&self) -> Document {
		PrettyPrint::render(self.as_ref())
	}

	fn pretty_print(&self, f: &mut Formatter<'_>) -> FmtResult {
		PrettyPrint::pretty_print(self.as_ref(), f)
	}
}

impl<T: PrettyPrint> PrettyPrint for [T] {
	fn render(&self) -> Document {
		let single = self.iter().fold(Document::Empty, |acc, e| match acc {
			Document::Empty => e.render(),
			acc => acc + ',' + ' ' + e.render(),
		});
		let multi = self.iter().fold(Document::Empty, |acc, e| match acc {
			Document::Empty => e.render(),
			acc => acc + ',' + nl() + e.render(),
		});
		let single_line = '[' + single + ']';
		let multi_line = '[' + indent(4, nl() + multi) + nl() + ']';
		single_line | multi_line
	}
}

impl<T: PrettyPrint> PrettyPrint for alloc::vec::Vec<T> {
	fn render(&self) -> Document {
		(**self).render()
	}
}

impl<T: PrettyPrint> PrettyPrint for alloc::collections::BTreeSet<T> {
	fn render(&self) -> Document {
		let single = self.iter().fold(Document::Empty, |acc, e| match acc {
			Document::Empty => e.render(),
			acc => acc + ',' + ' ' + e.render(),
		});
		let multi = self.iter().fold(Document::Empty, |acc, e| match acc {
			Document::Empty => e.render(),
			acc => acc + ',' + nl() + e.render(),
		});
		let single_line = '{' + single + '}';
		let multi_line = '{' + indent(4, nl() + multi) + nl() + '}';
		single_line | multi_line
	}
}

impl<K: PrettyPrint, V: PrettyPrint> PrettyPrint for alloc::collections::BTreeMap<K, V> {
	fn render(&self) -> Document {
		let single = self.iter().fold(Document::Empty, |acc, (k, v)| match acc {
			Document::Empty => k.render() + " => " + v.render(),
			acc => acc + ',' + ' ' + k.render() + " => " + v.render(),
		});
		let multi = self.iter().fold(Document::Empty, |acc, (k, v)| match acc {
			Document::Empty => k.render() + " => " + v.render(),
			acc => acc + ',' + nl() + k.render() + " => " + v.render(),
		});

		let single_line = '{' + single + '}';
		let multi_line = '{' + indent(4, nl() + multi) + nl() + '}';
		single_line | multi_line
	}
}

impl Display for dyn PrettyPrint {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		PrettyPrint::pretty_print(self, f)
	}
}

#[macro_export]
macro_rules! pretty_via_display {
	($name:ty) => {
		impl $crate::prettier::PrettyPrint for $name {
			fn render(&self) -> $crate::prettier::Document {
				$crate::prettier::display(*self)
			}
		}
	};
    ($($name:ty)*) => {
        $(
            pretty_via_display!($name);
        )*
    }
}

#[macro_export]
macro_rules! pretty_via_to_string {
	($name:ty) => {
		impl $crate::prettier::PrettyPrint for $name {
			fn render(&self) -> $crate::prettier::Document {
				$crate::prettier::text(&**self)
			}
		}
	};
}

pretty_via_display! {
	bool u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize core::num::NonZeroU8 core::num::NonZeroI8 core::num::NonZeroU16 core::num::NonZeroI16
	core::num::NonZeroU32 core::num::NonZeroI32 core::num::NonZeroU64 core::num::NonZeroI64 core::num::NonZeroU128 core::num::NonZeroI128
	core::num::NonZeroUsize core::num::NonZeroIsize
}
