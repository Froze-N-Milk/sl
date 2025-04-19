use std::ops::ControlFlow;

use super::{error, ParseResult, Parser, View};

impl<'buf> Parser<'buf> for &'buf str {
	type Output = &'buf str;
	type Error = (View<'buf>, &'static str, &'buf str);

	fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
		let found = self
			.char_indices()
			.zip(buf.as_str().char_indices())
			.try_fold(0, |found, ((i_l, c_l), (i_r, c_r))| {
				if c_l == c_r && i_l == i_r {
					return ControlFlow::Continue(i_l + c_l.len_utf8());
				} else {
					return ControlFlow::Break(found);
				}
			});

		let found = match found {
			ControlFlow::Continue(x) => x,
			ControlFlow::Break(x) => x,
		};
		return if found == self.len() {
			Ok((buf.sub_view(found..), &buf.as_str()[..found]))
		} else {
			Err((buf.sub_view(found..), "\nexpected: ", self))
		};
	}
}

#[test]
fn exact_match<'buf>() {
	let parser = "abc";
	let buf = View::new("abc");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, res) = res.unwrap();
	assert_eq!(("", "abc"), (buf.as_str(), res))
}

#[test]
fn totally_different<'buf>() {
	let parser = "abc";
	let buf = View::new("defgh");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_err());
}

#[test]
fn starts_with<'buf>() {
	let parser = "abc";
	let buf = View::new("abcd");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, res) = res.unwrap();
	assert_eq!(("d", "abc"), (buf.as_str(), res))
}

#[test]
fn ends_early<'buf>() {
	let parser = "abc";
	let buf = View::new("ab");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_err());
}

#[test]
fn partial<'buf>() {
	let parser = "abc";
	let buf = View::new("abd");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_err());
}

/// passes (captured string so far, next character) to F
/// and polls it until it returns false
/// this is infallible, and will return the empty string if it can't find anything
pub struct CaptureWhile<F: Fn(&str, char) -> bool>(pub F);

impl<'buf, F: Fn(&str, char) -> bool> Parser<'buf> for CaptureWhile<F> {
	type Output = &'buf str;
	type Error = error::Infallible;

	fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
		let mut chars = buf.as_str().char_indices();
		let capture;
		loop {
			match chars.next() {
				Some((i, c)) => {
					if !self.0(&buf.as_str()[..i], c) {
						capture = i;
						break;
					}
				}
				None => {
					capture = buf.as_str().len();
					break;
				}
			}
		}
		return Ok((buf.sub_view(capture..), &buf.as_str()[..capture]));
	}
}

#[test]
fn capture_all_whitespace<'buf>() {
	let parser = CaptureWhile(|_, char| char.is_whitespace());
	let buf = View::new("   ");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, res) = res.unwrap();
	assert_eq!(("", "   "), (buf.as_str(), res))
}

#[test]
fn capture_nothing<'buf>() {
	let parser = CaptureWhile(|_, char| !char.is_whitespace());
	let buf = View::new("   ");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, res) = res.unwrap();
	assert_eq!(("   ", ""), (buf.as_str(), res))
}

#[test]
fn capture_a<'buf>() {
	let parser = CaptureWhile(|_, char| char == 'a');
	let buf = View::new("abc");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, res) = res.unwrap();
	assert_eq!(("bc", "a"), (buf.as_str(), res))
}
