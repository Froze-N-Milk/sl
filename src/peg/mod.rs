//! PEG Parsers are monadically composable parsing expression grammars

pub use core::convert::Infallible;
pub mod graphemes;

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    #[inline(always)]
    pub const fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }
    #[inline(always)]
    pub const fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
}
impl<T> Either<T, T> {
    #[inline(always)]
    pub fn fuse(self) -> T {
        match self {
            Either::Left(t) => t,
            Either::Right(t) => t,
        }
    }
}

pub trait PEGParser<BUF: Copy, T, E> {
    fn parse(&self, buf: BUF) -> Result<(BUF, T), E>;
}

impl<BUF: Copy, T, E, F> PEGParser<BUF, T, E> for F
where
    F: Fn(BUF) -> Result<(BUF, T), E>,
{
    #[inline(always)]
    fn parse(&self, buf: BUF) -> Result<(BUF, T), E> {
        self(buf)
    }
}

/// performs some one-shot transformation, like parser but self-consuming
pub trait PEGAdaptor<BUF: Copy, T, E> {
    fn adapt(self, buf: BUF) -> Result<(BUF, T), E>;
}

impl<BUF: Copy, T, E, F> PEGAdaptor<BUF, T, E> for F
where
    F: FnOnce(BUF) -> Result<(BUF, T), E> + Sized,
{
    #[inline(always)]
    fn adapt(self, buf: BUF) -> Result<(BUF, T), E> {
        self(buf)
    }
}

pub trait PEGParserExt<BUF: Copy, T, E>: PEGParser<BUF, T, E> + Sized {
    #[inline(always)]
    fn flatmap<T2, E2, Adaptor: PEGAdaptor<BUF, T2, E2>>(
        self,
        f: impl Fn(Result<T, E>) -> Adaptor,
    ) -> impl PEGParser<BUF, T2, E2> {
        move |buf: BUF| -> Result<(BUF, T2), E2> {
            match self.parse(buf.clone()) {
                Ok((buf, res)) => f(Ok(res)).adapt(buf),
                Err(err) => f(Err(err)).adapt(buf),
            }
        }
    }

    #[inline(always)]
    fn then<T2, E2>(
        self,
        then: impl PEGParser<BUF, T2, E2>,
    ) -> impl PEGParser<BUF, (T, T2), Either<E, E2>> {
        (self, then)
    }

    #[inline(always)]
    fn then_left<T2, E2>(
        self,
        then: impl PEGParser<BUF, T2, E2>,
    ) -> impl PEGParser<BUF, T, Either<E, E2>> {
        (self, then).map(|(l, _)| l)
    }

    #[inline(always)]
    fn then_right<T2, E2>(
        self,
        then: impl PEGParser<BUF, T2, E2>,
    ) -> impl PEGParser<BUF, T2, Either<E, E2>> {
        (self, then).map(|(_, r)| r)
    }

    #[inline(always)]
    fn or<T2, E2>(
        self,
        or: impl PEGParser<BUF, T2, E2>,
    ) -> impl PEGParser<BUF, Either<T, T2>, (E, E2)> {
        move |buf| match self.parse(buf) {
            Ok((buf, res)) => Ok((buf, Either::Left(res))),
            Err(e) => or
                .parse(buf)
                .map(|(buf, res)| (buf, Either::Right(res)))
                .map_err(|e2| (e, e2)),
        }
    }

    #[inline(always)]
    fn and_then<T2>(self, f: impl Fn(T) -> Result<T2, E>) -> impl PEGParser<BUF, T2, E> {
        move |buf: BUF| match self.parse(buf) {
            Ok((buf, res)) => f(res).map(|res| (buf, res)),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    fn or_else<E2>(self, f: impl Fn(E) -> Result<T, E2>) -> impl PEGParser<BUF, T, E2> {
        move |buf: BUF| match self.parse(buf.clone()) {
            Err(err) => f(err).map(|res| (buf, res)),
            Ok(res) => Ok(res),
        }
    }

    #[inline(always)]
    fn optional(self) -> impl PEGParser<BUF, Option<T>, Infallible> {
        self.flatmap(move |res| move |buf| Ok((buf, res.ok())))
    }

    #[inline(always)]
    fn map<T2, F: Fn(T) -> T2 + Copy>(self, f: F) -> impl PEGParser<BUF, T2, E> {
        self.flatmap(move |res| move |buf| res.map(|res| (buf, f(res))))
    }

    #[inline(always)]
    fn map_err<E2, F: Fn(E) -> E2 + Copy>(self, f: F) -> impl PEGParser<BUF, T, E2> {
        self.flatmap(move |res| move |buf| res.map_err(|err| f(err)).map(|res| (buf, res)))
    }

    #[inline(always)]
    fn greedy(self) -> impl PEGParser<BUF, (Vec<T>, E), Infallible> {
        move |buf| {
            let mut vec = Vec::new();
            let (buf, err) = greedy_rec(buf, &self, &mut vec);
            Ok((buf, (vec, err)))
        }
    }
}

#[inline(always)]
fn greedy_rec<BUF: Copy, T, E>(
    buf: BUF,
    parser: &impl PEGParser<BUF, T, E>,
    vec: &mut Vec<T>,
) -> (BUF, E) {
    let (buf, res) = match parser.parse(buf.clone()) {
        Ok(x) => x,
        Err(err) => return (buf, err),
    };

    vec.push(res);
    greedy_rec(buf, parser, vec)
}

impl<BUF: Copy, T, E, PEG> PEGParserExt<BUF, T, E> for PEG where PEG: PEGParser<BUF, T, E> {}

impl<BUF: Copy, T, T2, E, E2, A, B> PEGParser<BUF, (T, T2), Either<E, E2>> for (A, B)
where
    A: PEGParser<BUF, T, E>,
    B: PEGParser<BUF, T2, E2>,
{
    #[inline(always)]
    fn parse(&self, buf: BUF) -> Result<(BUF, (T, T2)), Either<E, E2>> {
        let (buf, a) = self.0.parse(buf).map_err(Either::Left)?;
        let (buf, b) = self.1.parse(buf).map_err(Either::Right)?;
        Ok((buf, (a, b)))
    }
}

pub fn debug<BUF: Copy + std::fmt::Debug, T: std::fmt::Debug, E: std::fmt::Debug>(
    msg: &'static str,
    peg: impl PEGParser<BUF, T, E>,
) -> impl PEGParser<BUF, T, E> {
    move |buf| {
        eprintln!("before {}: {:?}", msg, buf);
        let res = peg.parse(buf);
        match &res {
            Ok((buf, res)) => {
                eprintln!("after {}: {:?}", msg, buf);
                eprintln!("success {}: {:?}", msg, res);
            }
            Err(err) => {
                eprintln!("failure {}: {:?}", msg, err);
            }
        }
        res
    }
}
