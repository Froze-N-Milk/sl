use super::{ErrorMessage, ParseResult, Parser, View};

pub struct Map<P, F> {
    parser: P,
    f: F,
}

pub fn map<P, F>(parser: P, f: F) -> Map<P, F> {
    Map { parser, f }
}

impl<'buf, T, E, E2, T2, P, F> Parser<'buf> for Map<P, F>
where
    P: Parser<'buf, Output = T, Error = E>,
    E: ErrorMessage,
    E2: ErrorMessage,
    F: Fn(Result<T, E>) -> Result<T2, E2>,
{
    type Output = T2;
    type Error = E2;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        match self.parser.parse(buf.clone()) {
            Ok((buf, res)) => (self.f)(Ok(res)).map(|res| (buf, res)),
            Err(err) => (self.f)(Err(err)).map(|res| (buf, res)),
        }
    }
}

pub struct MapOk<P, F> {
    parser: P,
    f: F,
}

pub fn map_ok<P, F>(parser: P, f: F) -> MapOk<P, F> {
    MapOk { parser, f }
}

impl<'buf, T, E, T2, P, F> Parser<'buf> for MapOk<P, F>
where
    E: ErrorMessage,
    P: Parser<'buf, Output = T, Error = E>,
    F: Fn(T) -> Result<T2, E>,
{
    type Output = T2;
    type Error = E;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        let (buf, res) = self.parser.parse(buf)?;
        (self.f)(res).map(|res| (buf, res))
    }
}

pub struct MapErr<P, F> {
    parser: P,
    f: F,
}

pub fn map_err<P, F>(parser: P, f: F) -> MapErr<P, F> {
    MapErr { parser, f }
}

impl<'buf, T, E, E2, P, F> Parser<'buf> for MapErr<P, F>
where
    E: ErrorMessage,
    E2: ErrorMessage,
    P: Parser<'buf, Output = T, Error = E>,
    F: Fn(E) -> Result<T, E2>,
{
    type Output = T;
    type Error = E2;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        match self.parser.parse(buf) {
            Ok(ok) => Ok(ok),
            Err(err) => (self.f)(err).map(|res| (buf, res)),
        }
    }
}
