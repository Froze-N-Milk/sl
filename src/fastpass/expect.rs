use super::{ErrorMessage, ParseResult, Parser, View};

pub struct Expect<F> {
    test: F
}

pub fn expect<'buf, E: ErrorMessage, F: Fn(View<'buf>) -> Option<E>>(test: F) -> Expect<F> {
    Expect { test }
}

impl <'buf, E: ErrorMessage, F: Fn(View<'buf>) -> Option<E>> Parser<'buf> for Expect<F> {
    type Output = ();
    type Error = E;

    fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
        (self.test)(buf).map_or(Ok((buf, ())), |err| Err(err))
    }
}
