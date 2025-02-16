pub type ParseResult<BUF, T, E> = Result<(BUF, T), E>;

mod flatmap;
pub use flatmap::{FlatMap, FlatMapErr, FlatMapOk};

mod map;
pub use map::{Map, MapErr, MapOk};

pub mod graphemes;

mod buffer;
pub use buffer::*;

pub enum ParserErr<F, N> {
    Fatal(F),
    NonFatal(N),
}

pub trait Parser<BUF: Buffer>: Sized {
    type Output;
    type Error;
    fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error>;
    fn flatmap<
        T2,
        E2,
        P: Parser<BUF, Output = T2, Error = E2>,
        F: Fn(Result<Self::Output, Self::Error>) -> P,
    >(
        self,
        f: F,
    ) -> FlatMap<Self, F> {
        flatmap::flatmap(self, f)
    }
    fn flatmap_ok<
        T2,
        P: Parser<BUF, Output = T2, Error = Self::Error>,
        F: Fn(Self::Output) -> P,
    >(
        self,
        f: F,
    ) -> FlatMapOk<Self, F> {
        flatmap::flatmap_ok(self, f)
    }
    fn flatmap_err<
        E2,
        P: Parser<BUF, Output = Self::Output, Error = E2>,
        F: Fn(Self::Error) -> P,
    >(
        self,
        f: F,
    ) -> FlatMapErr<Self, F> {
        flatmap::flatmap_err(self, f)
    }

    fn map<T2, E2, F: Fn(Result<Self::Output, Self::Error>) -> Result<T2, E2>>(
        self,
        f: F,
    ) -> Map<Self, F> {
        map::map(self, f)
    }
    fn map_ok<T2, F: Fn(Self::Output) -> Result<T2, Self::Error>>(self, f: F) -> MapOk<Self, F> {
        map::map_ok(self, f)
    }
    fn map_err<E2, F: Fn(Self::Error) -> Result<Self::Output, E2>>(self, f: F) -> MapErr<Self, F> {
        map::map_err(self, f)
    }
}

macro_rules! impl_parser_for_tuple {
  ($($parser:ident $output:ident),+) => (
    #[allow(non_snake_case)]
    impl<BUF: Buffer, $($output),+, E, $($parser),+> Parser<BUF> for ($($parser),+,)
    where
      $($parser: Parser<BUF, Output = $output, Error = E>),+
    {
      type Output = ($($output),+,);
      type Error = E;

      #[inline(always)]
      fn parse(&self, buf: BUF) -> ParseResult<BUF, Self::Output, Self::Error> {
        let ($(ref $parser),+,) = *self;

        $(let(buf, $output) = $parser.parse(buf)?;)+

        Ok((buf, ($($output),+,)))
      }
    }
  )
}

macro_rules! impl_parser_for_tuples {
    ($parser1:ident $output1:ident, $($parser:ident $output:ident),+) => {
        impl_parser_for_tuples!(__impl $parser1 $output1; $($parser $output),+);
    };
    (__impl $($parser:ident $output:ident),+; $parser1:ident $output1:ident $(,$parser2:ident $output2:ident)*) => {
        impl_parser_for_tuple!($($parser $output),+);
        impl_parser_for_tuples!(__impl $($parser $output),+, $parser1 $output1; $($parser2 $output2),*);
    };
    (__impl $($parser:ident $output:ident),+;) => {
        impl_parser_for_tuple!($($parser $output),+);
    }
}

impl_parser_for_tuples!(P1 O1, P2 O2, P3 O3, P4 O4, P5 O5, P6 O6, P7 O7, P8 O8, P9 O9, P10 O10, P11 O11, P12 O12, P13 O13, P14 O14, P15 O15, P16 O16, P17 O17, P18 O18, P19 O19, P20 O20, P21 O21);
