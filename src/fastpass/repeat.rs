use super::{Infallible, Parser, View};

pub struct Greedy<P> {
	parser: P,
}

pub fn greedy<P>(parser: P) -> Greedy<P> {
	Greedy { parser }
}

impl<'buf, P: Parser<'buf>> Parser<'buf> for Greedy<P> {
	type Output = (Vec<P::Output>, P::Error);
	type Error = Infallible;

	fn parse(&self, mut buf: View<'buf>) -> Result<(View<'buf>, Self::Output), Self::Error> {
		let mut v = Vec::new();
		loop {
			match self.parser.parse(buf) {
				Ok((buf2, res)) => {
					v.push(res);
					buf = buf2;
				}
				Err(err) => return Ok((buf, (v, err))),
			}
		}
	}
}

pub struct RepeatIf<P, F> {
	parser: P,
	test: F,
}

pub fn repeat_if<P, F>(parser: P, test: F) -> RepeatIf<P, F> {
	RepeatIf { parser, test }
}

impl<'buf, P: Parser<'buf>, F: Fn(&Vec<P::Output>) -> bool> Parser<'buf> for RepeatIf<P, F> {
	type Output = Vec<P::Output>;
	type Error = P::Error;

	fn parse(&self, mut buf: View<'buf>) -> Result<(View<'buf>, Self::Output), Self::Error> {
		let mut v = Vec::new();
		loop {
			match self.parser.parse(buf) {
				Ok((buf2, res)) => {
					v.push(res);
					buf = buf2;
					if !(self.test)(&v) {
						return Ok((buf, v));
					}
				}
				Err(err) => return Err(err),
			}
		}
	}
}
