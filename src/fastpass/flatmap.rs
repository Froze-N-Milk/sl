use super::{ParseResult, Parser};

pub struct FlatMap<P, F> {
    parser: P,
    f: F,
}

pub fn flatmap<P, F>(parser: P, f: F) -> FlatMap<P, F> {
    FlatMap { parser, f }
}

impl<BUF, T, T2, E, E2, P, F, P2> Parser<BUF> for FlatMap<P, F>
where
    P: Parser<BUF, Output = T, Error = E>,
    F: Fn(Result<T, E>) -> P2,
    P2: Parser<BUF, Output = T2, Error = E2>,
    BUF: Clone,
{
    type Output = P2::Output;

    type Error = P2::Error;

    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
        match self.parser.parse(buf.clone()) {
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

impl<BUF, T, T2, E, P, F, P2> Parser<BUF> for FlatMapOk<P, F>
where
    P: Parser<BUF, Output = T, Error = E>,
    F: Fn(T) -> P2,
    P2: Parser<BUF, Output = T2, Error = E>,
    BUF: Clone,
{
    type Output = P2::Output;

    type Error = P2::Error;

    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
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

impl<BUF, T, E, E2, P, F, P2> Parser<BUF> for FlatMapErr<P, F>
where
    P: Parser<BUF, Output = T, Error = E>,
    F: Fn(E) -> P2,
    P2: Parser<BUF, Output = T, Error = E2>,
    BUF: Clone,
{
    type Output = P2::Output;

    type Error = P2::Error;

    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
        match self.parser.parse(buf.clone()) {
            Ok(ok) => Ok(ok),
            Err(err) => (self.f)(err).parse(buf),
        }
    }
}
