use super::{ParseResult, Parser, Buffer};

impl <'buf> Parser<&'buf str> for &'buf str {
    type Output = &'buf str;

    type Error = (); // TODO

    fn parse(&self, buf: &'buf str) -> ParseResult<&'buf str, Self::Output, Self::Error> {
        todo!()
    }
}
