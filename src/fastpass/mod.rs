pub type ParseResult<'buf, T, E> = Result<(View<'buf>, T), E>;

mod flatmap;
pub use flatmap::{FlatMap, FlatMapErr, FlatMapOk};

mod map;
pub use map::{Map, MapErr, MapOk};

mod optional;
pub use optional::Optional;

mod then;
pub use then::Then;

mod or;
pub use or::Or;

mod repeat;
pub use repeat::{Greedy, RepeatIf};

mod expect;
pub use expect::Expect;

mod str;
pub use str::*;

mod view;
pub use view::*;

mod error;
pub use error::*;

mod either;
pub use either::*;

pub trait Parser<'buf>: Sized {
	type Output;
	type Error: ErrorMessage;

	fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error>;
	fn flatmap<
		T2,
		P: Parser<'buf, Output = T2>,
		F: Fn(Result<Self::Output, Self::Error>) -> P,
	>(
		self,
		f: F,
	) -> FlatMap<Self, F> {
		flatmap::flatmap(self, f)
	}
	fn flatmap_ok<T2, P: Parser<'buf, Output = T2>, F: Fn(Self::Output) -> P>(
		self,
		f: F,
	) -> FlatMapOk<Self, F> {
		flatmap::flatmap_ok(self, f)
	}
	fn flatmap_err<P: Parser<'buf, Output = Self::Output>, F: Fn(Self::Error) -> P>(
		self,
		f: F,
	) -> FlatMapErr<Self, F> {
		flatmap::flatmap_err(self, f)
	}

	fn map<T, E: ErrorMessage, F: Fn(Result<Self::Output, Self::Error>) -> Result<T, E>>(
		self,
		f: F,
	) -> Map<Self, F> {
		map::map(self, f)
	}
	fn map_ok<T, F: Fn(Self::Output) -> Result<T, Self::Error>>(self, f: F) -> MapOk<Self, F> {
		map::map_ok(self, f)
	}
	fn map_err<E: ErrorMessage, F: Fn(Self::Error) -> Result<Self::Output, E>>(
		self,
		f: F,
	) -> MapErr<Self, F> {
		map::map_err(self, f)
	}

	fn optional(self) -> Optional<Self> {
		optional::optional(self)
	}

	fn or<P: Parser<'buf>>(self, or: P) -> Or<Self, P> {
		or::or(self, or)
	}

	fn then<P: Parser<'buf>>(self, then: P) -> Then<Self, P> {
		then::then(self, then)
	}
	fn then_left<P: Parser<'buf>>(
		self,
		then: P,
	) -> MapOk<
		Then<Self, P>,
		impl Fn(
			(Self::Output, P::Output),
		) -> Result<Self::Output, Either<Self::Error, P::Error>>,
	> {
		then::then(self, then).map_ok(|(l, _)| Ok(l))
	}
	fn then_right<P: Parser<'buf>>(
		self,
		then: P,
	) -> MapOk<
		Then<Self, P>,
		impl Fn((Self::Output, P::Output)) -> Result<P::Output, Either<Self::Error, P::Error>>,
	> {
		then::then(self, then).map_ok(|(_, r)| Ok(r))
	}

	fn greedy(self) -> Greedy<Self> {
		repeat::greedy(self)
	}

	fn repeat_if<F: Fn(&Vec<Self::Output>) -> bool>(self, test: F) -> RepeatIf<Self, F> {
		repeat::repeat_if(self, test)
	}

	fn expect<E: ErrorMessage, F: Fn(View<'buf>) -> Option<E>>(
		self,
		test: F,
	) -> impl Parser<
		'buf,
		Output = Self::Output,
		Error = Either<Self::Error, <Expect<F> as Parser<'buf>>::Error>,
	> {
		self.then_left(expect::expect(test))
	}
}

impl<'buf, T, E: ErrorMessage, F: Fn(View<'buf>) -> ParseResult<'buf, T, E>> Parser<'buf> for F {
	type Output = T;
	type Error = E;

	fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
		self(buf)
	}
}

//macro_rules! impl_parser_for_tuple {
//  ($($parser:ident $output:ident),+) => (
//    #[allow(non_snake_case)]
//    impl<'buf, $($output),+, $($parser),+> Parser<'buf> for ($($parser),+,)
//    where
//      $($parser: Parser<'buf, Output = $output>),+
//    {
//      type Output = ($($output),+,);
//
//      #[inline(always)]
//      fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output> {
//        let ($(ref $parser),+,) = *self;
//
//        $(let(buf, $output) = $parser.parse(buf)?;)+
//
//        Ok((buf, ($($output),+,)))
//      }
//    }
//  )
//}
//
//macro_rules! impl_parser_for_tuples {
//    ($parser1:ident $output1:ident, $($parser:ident $output:ident),+) => {
//        impl_parser_for_tuples!(__impl $parser1 $output1; $($parser $output),+);
//    };
//    (__impl $($parser:ident $output:ident),+; $parser1:ident $output1:ident $(,$parser2:ident $output2:ident)*) => {
//        impl_parser_for_tuple!($($parser $output),+);
//        impl_parser_for_tuples!(__impl $($parser $output),+, $parser1 $output1; $($parser2 $output2),*);
//    };
//    (__impl $($parser:ident $output:ident),+;) => {
//        impl_parser_for_tuple!($($parser $output),+);
//    }
//}
//
//impl_parser_for_tuples!(P1 O1, P2 O2, P3 O3, P4 O4, P5 O5, P6 O6, P7 O7, P8 O8, P9 O9, P10 O10, P11 O11, P12 O12, P13 O13, P14 O14, P15 O15, P16 O16, P17 O17, P18 O18, P19 O19, P20 O20, P21 O21);
//

//macro_rules! impl_parser_for_tuple {
//  ($($parser:ident $output:ident $error:ident),+) => (
//    #[allow(non_snake_case)]
//    impl<'buf, $($output),+, $($error),+, $($parser),+> Parser<'buf> for ($($parser),+,)
//    where
//      $($parser: Parser<'buf, Output = $output>),+
//    {
//      type Output = ($($output),+,);
//      type Error = ($($error),+,);
//
//      #[inline(always)]
//      fn parse(&self, buf: View<'buf>) -> ParseResult<'buf, Self::Output, Self::Error> {
//        let ($(ref $parser),+,) = *self;
//
//        $(let(buf, $output) = $parser.parse(buf)?;)+
//
//        Ok((buf, ($($output),+,)))
//      }
//    }
//  )
//}
//
//macro_rules! impl_parser_for_tuples {
//    ($parser1:ident $output1:ident $error1:ident, $($parser:ident $output:ident $error:ident),+) => {
//        impl_parser_for_tuples!(__impl $parser1 $output1 $error1; $($parser $output $error),+);
//    };
//    (__impl $($parser:ident $output:ident $error:ident),+; $parser1:ident $output1:ident $error1:ident $(,$parser2:ident $output2:ident $error2:ident)*) => {
//        impl_parser_for_tuple!($($parser $output $error),+);
//        impl_parser_for_tuples!(__impl $($parser $output $error),+, $parser1 $output1 $error1; $($parser2 $output2 $error2),*);
//    };
//    (__impl $($parser:ident $output:ident $error:ident),+;) => {
//        impl_parser_for_tuple!($($parser $output $error),+);
//    }
//}
//
//impl_parser_for_tuples!(P1 O1 E1, P2 O2 E2, P3 O3 E3, P4 O4 E4, P5 O5 E5, P6 O6 E6, P7 O7 E7, P8 O8 E8, P9 O9 E9, P10 O10 E10, P11 O11 E11, P12 O12 E12, P13 O13 E13, P14 O14 E14, P15 O15 E15, P16 O16 E16, P17 O17 E17, P18 O18 E18, P19 O19 E19, P20 O20 E20, P21 O21 E21);
