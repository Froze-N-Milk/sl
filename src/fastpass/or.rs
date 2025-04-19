use super::{Either, Parser, View};

pub struct Or<L, R> {
    l: L,
    r: R,
}

pub fn or<L, R>(l: L, r: R) -> Or<L, R> {
    Or { l, r }
}

impl <'buf, L: Parser<'buf>, R: Parser<'buf>> Parser<'buf> for Or<L, R> {
    type Output = Either<L::Output, R::Output>;
    type Error = (L::Error, R::Error);

    fn parse(&self, buf: View<'buf>) -> Result<(View<'buf>, Self::Output), Self::Error> {
        match self.l.parse(buf) {
            Ok((buf, res)) => Ok((buf, Either::L(res))),
            Err(l_err) => match self.r.parse(buf) {
                Ok((buf, res)) => Ok((buf, Either::R(res))),
                Err(r_err) => Err((l_err, r_err)),
            },
        }
    }
}

