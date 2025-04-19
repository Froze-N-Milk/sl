use super::{Infallible, ParseResult, Parser};

pub struct Optional<P> {
	parser: P,
}

pub fn optional<P>(parser: P) -> Optional<P> {
	Optional { parser }
}

impl<'buf, P: Parser<'buf>> Parser<'buf> for Optional<P> {
	type Output = Option<P::Output>;
	type Error = Infallible;

	fn parse(&self, buf: super::View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
		self.parser
			.parse(buf)
			.map_or(Ok((buf, None)), |(buf, res)| Ok((buf, Some(res))))
	}
}
