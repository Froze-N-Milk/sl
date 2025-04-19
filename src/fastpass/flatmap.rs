use super::{ErrorMessage, ParseResult, Parser, View};

pub struct FlatMap<P, F> {
    parser: P,
    f: F,
}

pub fn flatmap<P, F>(parser: P, f: F) -> FlatMap<P, F> {
    FlatMap { parser, f }
}

impl<'buf, T, E, T2, E2, P, F, P2> Parser<'buf> for FlatMap<P, F>
where
    E: ErrorMessage,
    E2: ErrorMessage,
    P: Parser<'buf, Output = T, Error = E>,
    F: Fn(Result<T, E>) -> P2,
    P2: Parser<'buf, Output = T2, Error = E2>,
{
    type Output = P2::Output;
    type Error = P2::Error;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        match self.parser.parse(buf) {
            Ok((buf, res)) => (self.f)(Ok(res)).parse(buf),
            Err(err) => (self.f)(Err(err)).parse(buf),
        }
    }
}

pub struct FlatMapOk<P, F> {
    parser: P,
    f: F,
}

pub fn flatmap_ok<P, F>(parser: P, f: F) -> FlatMapOk<P, F> {
    FlatMapOk { parser, f }
}

impl<'buf, T, E, T2, P, F, P2> Parser<'buf> for FlatMapOk<P, F>
where
    E: ErrorMessage,
    P: Parser<'buf, Output = T, Error = E>,
    F: Fn(T) -> P2,
    P2: Parser<'buf, Output = T2, Error = E>,
{
    type Output = P2::Output;
    type Error = P2::Error;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        let (buf, res) = self.parser.parse(buf)?;
        (self.f)(res).parse(buf)
    }
}

pub struct FlatMapErr<P, F> {
    parser: P,
    f: F,
}

pub fn flatmap_err<P, F>(parser: P, f: F) -> FlatMapErr<P, F> {
    FlatMapErr { parser, f }
}

impl<'buf, T, E, E2, P, F, P2> Parser<'buf> for FlatMapErr<P, F>
where
    E: ErrorMessage,
    E2: ErrorMessage,
    P: Parser<'buf, Output = T, Error = E>,
    F: Fn(E) -> P2,
    P2: Parser<'buf, Output = T, Error = E2>,
{
    type Output = P2::Output;
    type Error = P2::Error;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        match self.parser.parse(buf.clone()) {
            Ok(ok) => Ok(ok),
            Err(err) => (self.f)(err).parse(buf),
        }
    }
}
