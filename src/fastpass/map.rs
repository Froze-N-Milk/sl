use super::{ParseResult, Parser};

pub struct Map<P, F> {
    parser: P,
    f: F,
}

pub fn map<P, F>(parser: P, f: F) -> Map<P, F> {
    Map { parser, f }
}

impl<BUF, T, T2, E, E2, P, F> Parser<BUF> for Map<P, F>
where
    P: Parser<BUF, Output = T, Error = E>,
    F: Fn(Result<T, E>) -> Result<T2, E2>,
    BUF: Clone,
{
    type Output = T2;

    type Error = E2;

    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
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

impl<BUF, T, T2, E, P, F> Parser<BUF> for MapOk<P, F>
where
    P: Parser<BUF, Output = T, Error = E>,
    F: Fn(T) -> Result<T2, E>,
    BUF: Clone,
{
    type Output = T2;

    type Error = E;

    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
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

impl<BUF, T, E, E2, P, F> Parser<BUF> for MapErr<P, F>
where
    P: Parser<BUF, Output = T, Error = E>,
    F: Fn(E) -> Result<T, E2>,
    BUF: Clone,
{
    type Output = T;

    type Error = E2;

    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
        match self.parser.parse(buf.clone()) {
            Ok(ok) => Ok(ok),
            Err(err) => (self.f)(err).map(|res| (buf, res)),
        }
    }
}
