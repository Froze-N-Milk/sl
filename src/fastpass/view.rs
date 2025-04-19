use core::{
	fmt,
	ops::{Bound, RangeBounds},
};

use crate::fastpass::ErrorMessage;

pub struct View<'buf> {
	source: &'buf str,
	pub start: usize,
	pub end: usize,
}

impl<'buf> fmt::Debug for View<'buf> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\"{}\"", self.as_str())
	}
}

impl<'buf> fmt::Display for View<'buf> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl<'buf> Clone for View<'buf> {
	fn clone(&self) -> Self {
		Self { ..*self }
	}
}

impl<'buf> Copy for View<'buf> {}

impl<'a, 'buf: 'a> View<'buf> {
	pub fn new(source: &'buf str) -> Self {
		Self {
			source,
			start: 0,
			end: source.len(),
		}
	}

	pub fn sub_view<R: RangeBounds<usize>>(&self, index: R) -> View<'buf> {
		let start = match index.start_bound() {
			Bound::Included(x) => self.start + x,
			Bound::Excluded(x) => self.start + x + 1,
			Bound::Unbounded => self.start,
		};
		let end = match index.end_bound() {
			Bound::Included(x) => self.start + x + 1,
			Bound::Excluded(x) => self.start + x,
			Bound::Unbounded => self.end,
		};

		Self {
			start,
			end,
			..*self
		}
	}

	pub fn as_str(&'a self) -> &'buf str {
		&self.source[self.start..self.end]
	}

	const ATTENTION: usize = 2;
}

impl<'buf> ErrorMessage for View<'buf> {
	fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let preceding = &self.source[..self.start];
		let row = preceding.lines().count();
		let col = preceding.lines().last().map_or(0, str::len);
		let out = self.source.lines().nth(if row == 0 { 0 } else { row - 1 }).unwrap_or("");
		let start = if col <= Self::ATTENTION {
			0
		} else {
			col - Self::ATTENTION - 1
		};
		write!(f, "{out}\n")?;
		for _ in 0..start {
			write!(f, " ")?;
		}
		let attention_count = if start <= Self::ATTENTION {
			Self::ATTENTION + Self::ATTENTION + 1 - (Self::ATTENTION - start)
		} else {
			Self::ATTENTION + Self::ATTENTION + 1
		};
		for _ in 0..attention_count {
			write!(f, "^")?;
		}
		write!(f, "\n")?;
		write!(f, "row: {row}, col: {col}\n")
	}
}

#[test]
fn sub_view1() {
    let v = View::new("12345");
    assert_eq!("12345", v.as_str());
    let v = v.sub_view(..4);
    assert_eq!("1234", v.as_str());
    let v = v.sub_view(1..);
    assert_eq!("234", v.as_str());
    let v = v.sub_view(1..2);
    assert_eq!("3", v.as_str());
}
