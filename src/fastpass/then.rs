use super::{Either, ParseResult, Parser, View};

pub struct Then<L, R> {
	l: L,
	r: R,
}

pub fn then<L, R>(l: L, r: R) -> Then<L, R> {
	Then { l, r }
}

impl<'buf, L: Parser<'buf>, R: Parser<'buf>> Parser<'buf> for Then<L, R> {
	type Output = (L::Output, R::Output);
	type Error = Either<L::Error, R::Error>;

	fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
		let (buf, l) = self.l.parse(buf).map_err(Either::L)?;
		let (buf, r) = self.r.parse(buf).map_err(Either::R)?;
		Ok((buf, (l, r)))
	}
}
