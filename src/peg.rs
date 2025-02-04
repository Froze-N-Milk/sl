//! PEG Parsers are monadically composable parsing expression grammars
use bytes::Buf;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    const fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }
    const fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
}
impl<T> Either<T, T> {
    fn fuse(self) -> T {
        match self {
            Either::Left(t) => t,
            Either::Right(t) => t,
        }
    }
}

pub enum Infallible {}

pub trait PEGParser<T, E> {
    fn parse(&self, buf: bytes::Bytes) -> Result<(bytes::Bytes, T), E>;
}

impl<T, E, F> PEGParser<T, E> for F
where
    F: Fn(bytes::Bytes) -> Result<(bytes::Bytes, T), E>,
{
    fn parse(&self, buf: bytes::Bytes) -> Result<(bytes::Bytes, T), E> {
        self(buf)
    }
}

/// performs some one-shot transformation, like parser but self-consuming
pub trait PEGAdaptor<T, E> {
    fn adapt(self, buf: bytes::Bytes) -> Result<(bytes::Bytes, T), E>;
}

impl<T, E, F> PEGAdaptor<T, E> for F
where
    F: FnOnce(bytes::Bytes) -> Result<(bytes::Bytes, T), E> + Sized,
{
    fn adapt(self, buf: bytes::Bytes) -> Result<(bytes::Bytes, T), E> {
        self(buf)
    }
}

pub trait PEGParserExt<T, E>: PEGParser<T, E> + Sized {
    #[inline(always)]
    fn flatmap<T2, E2, Adaptor: PEGAdaptor<T2, E2>>(
        self,
        f: impl Fn(Result<T, E>) -> Adaptor,
    ) -> impl PEGParser<T2, E2> {
        move |buf: bytes::Bytes| -> Result<(bytes::Bytes, T2), E2> {
            match self.parse(buf.clone()) {
                Ok((buf, res)) => f(Ok(res)).adapt(buf),
                Err(err) => f(Err(err)).adapt(buf),
            }
        }
    }

    #[inline(always)]
    fn then<T2>(self, then: impl PEGParser<T2, E>) -> impl PEGParser<(T, T2), E> {
        (self, then).map_err(Either::fuse)
    }

    #[inline(always)]
    fn map<T2, F: Fn(T) -> T2 + Copy>(self, f: F) -> impl PEGParser<T2, E> {
        self.flatmap(move |res|
                     move |buf|
                        res.map(|res| (buf, f(res))))
    }

    #[inline(always)]
    fn map_err<E2, F: Fn(E) -> E2 + Copy>(self, f: F) -> impl PEGParser<T, E2> {
        self.flatmap(move |res|
                     move |buf|
                         res.map_err(|err| f(err))
                            .map(|res| (buf, res)))
    }

    #[inline(always)]
    fn greedy(self) -> impl PEGParser<(Vec<T>, E), Infallible> {
        move |buf| {
            let mut vec = Vec::new();
            let (buf, err) = greedy_rec(buf, &self, &mut vec);
            Ok((buf, (vec, err)))
        }
    }
}

#[inline(always)]
fn greedy_rec<T, E>(buf: bytes::Bytes, parser: &impl PEGParser<T, E>, vec: &mut Vec<T>) -> (bytes::Bytes, E) {
    let (buf, res) = match parser.parse(buf.clone()) {
        Ok(x) => x,
        Err(err) => return (buf, err),
    };

    vec.push(res);
    greedy_rec(buf, parser, vec)
}

impl<T, E, PEG> PEGParserExt<T, E> for PEG where PEG: PEGParser<T, E> {}

impl<T, T2, E, E2, A, B> PEGParser<(T, T2), Either<E, E2>> for (A, B)
where
    A: PEGParser<T, E>,
    B: PEGParser<T2, E2>,
{
    #[inline(always)]
    fn parse(&self, buf: bytes::Bytes) -> Result<(bytes::Bytes, (T, T2)), Either<E, E2>> {
        let (buf, a) = self.0.parse(buf).map_err(Either::Left)?;
        let (buf, b) = self.1.parse(buf).map_err(Either::Right)?;
        Ok((buf, (a, b)))
    }
}

impl<'a> PEGParser<&'a str, ()> for &'a str {
    #[inline(always)]
    fn parse(&self, mut buf: bytes::Bytes) -> Result<(bytes::Bytes, &'a str), ()> {
        if self.len() > buf.remaining() {
            return Err(());
        }
        let remaining = buf.split_off(self.len());
        if buf == self.as_bytes() {
            return Ok((remaining, self));
        }
        Err(())
    }
}
