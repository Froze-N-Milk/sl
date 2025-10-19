use crate::frc::Frc;
use core::fmt;
use std::rc::Rc;

use crate::fastpass::{
    self, CaptureWhile, Either, ErrorMessage, Infallible, ParseResult, Parser, View,
};

use super::Syntax;

const SYMBOL_ILLEGALS: &[char] = &[' ', '\r', '\n', '\t', '(', ')', ';'];

#[inline(always)]
fn symbol<'buf>(buf: View<'buf>) -> ParseResult<'buf, Syntax, NoSymbol<'buf>> {
    let Ok((buf, res)) =
        fastpass::CaptureWhile(|_, char| !SYMBOL_ILLEGALS.contains(&char)).parse(buf);
    match res {
        "" => Err(NoSymbol(buf)),
        x => Ok((buf, Syntax::Sym(x.into()))),
    }
}

#[test]
fn symbol_test() {
    let parser = symbol;
    let buf = View::new("abc");
    let res = Parser::parse(&parser, buf);
    assert!(res.is_ok());
    let (buf, res) = res.unwrap();
    assert_eq!("", buf.as_str());
    match res {
        Syntax::Sym(res) => assert_eq!(*"abc", *res),
        _ => panic!(),
    }
}

#[inline(always)]
fn comment<'buf>(buf: View<'buf>) -> ParseResult<'buf, (), Infallible> {
    match ";".then(CaptureWhile(|_, c| c != '\n')).parse(buf) {
        Ok((buf, _)) => Ok((buf, ())),
        Err(_) => Ok((buf, ())),
    }
}

#[test]
fn comment_test() {
    let parser = comment;
    let buf = View::new(";    ");
    let res = Parser::parse(&parser, buf);
    assert!(res.is_ok());
    let (buf, _) = res.unwrap();
    assert_eq!("", buf.as_str());
}

#[inline(always)]
fn whitespace<'buf>(buf: View<'buf>) -> ParseResult<'buf, (), NoWhitespace<'buf>> {
    let Ok((buf, res)) =
        fastpass::CaptureWhile(|_, char| [' ', '\r', '\n', '\t'].contains(&char)).parse(buf);
    match res {
        "" => Err(NoWhitespace(buf)),
        _ => Ok((buf, ())),
    }
}

#[inline(always)]
fn swallow<'buf>(mut buf: View<'buf>) -> ParseResult<'buf, (), Infallible> {
    let p = whitespace.or(comment);
    loop {
        let Ok((buf2, _)) = p.parse(buf);
        if buf.as_str() == buf2.as_str() {
            return Ok((buf2, ()));
        }
        buf = buf2;
    }
}

//const NUMBER_LEGALS: &[char] = &['.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
//
//#[inline(always)]
//fn number<'buf>(buf: View<'buf>) -> ParseResult<'buf, Syntax, NoNumber<'buf>> {
//    let Ok((buf, res)) = fastpass::CaptureWhile(|_, char| NUMBER_LEGALS.contains(&char)).parse(buf);
//    match res.parse::<f64>() {
//        Ok(res) => Ok((buf, Syntax::Number(res))),
//        Err(_) => Err(NoNumber(buf)),
//    }
//}

//#[inline(always)]
//fn bool<'buf>(buf: View<'buf>) -> ParseResult<'buf, Value, NoBool<'buf>> {
//	match "#t".or("#f").parse(buf) {
//		Ok((buf, Either::L(_))) => {
//			Ok((buf, Value::new_unchecked(Type::Bool, Syntax::Bool(true))))
//		}
//		Ok((buf, Either::R(_))) => {
//			Ok((buf, Value::new_unchecked(Type::Bool, Syntax::Bool(false))))
//		}
//		Err(_) => Err(NoBool(buf)),
//	}
//}

#[inline(always)]
fn sexpr<'buf>(
    buf: View<'buf>,
) -> ParseResult<'buf, Syntax, Either<NoSExpr<'buf>, UnclosedSExpr<'buf>>> {
    let open = "(".map_err(|(buf, _, _)| Err(Either::L(NoSExpr(buf))));
    let close = ")".map_err(|(buf, _, _)| Err(Either::R(UnclosedSExpr(buf))));

    let (buf, _) = open.parse(buf)?;

    let Ok((buf, (exprs, Either::L(err)))) = expr.then_left(swallow).greedy().parse(buf);

    let exprs = match err {
        unclosed_sexpr @ Either::R(_) => return Err(unclosed_sexpr),
        _ => into_cons(exprs.into_iter()),
    };

    let (buf, _) = close.parse(buf)?;
    let Ok((buf, _)) = swallow.parse(buf);
    Ok((buf, exprs))
}

#[inline(always)]
fn expr<'buf>(
    buf: View<'buf>,
) -> ParseResult<'buf, Syntax, Either<NoSExpr<'buf>, UnclosedSExpr<'buf>>> {
    //match bool.then_left(swallow).parse(buf) {
    //	Ok(res) => return Ok(res),
    //	_ => (),
    //};
    match symbol.then_left(swallow).parse(buf) {
        Ok(res) => return Ok(res),
        _ => (),
    };
    match sexpr.then_left(swallow).parse(buf) {
        Ok(res) => Ok(res),
        Err(Either::L(err)) => Err(err),
    }
}

#[inline(always)]
pub fn sl<'buf>(
    buf: View<'buf>,
) -> Result<Rc<[Syntax]>, Either<UnclosedSExpr<'buf>, UnexpectedToken<'buf>>> {
    let Ok((buf, _)) = swallow.parse(buf);

    let Ok((buf, (exprs, err))) = expr.greedy().parse(buf);
    let exprs = match err {
        Either::R(err) => return Err(Either::L(err)),
        _ => exprs.into(),
    };

    match buf.as_str() {
        "" => Ok(exprs),
        _ => Err(Either::R(UnexpectedToken(buf))),
    }
}

fn into_cons(mut xs: impl Iterator<Item = Syntax>) -> Syntax {
    match xs.next() {
        Some(x) => Syntax::Cons(Frc::cons((x, into_cons(xs)))),
        None => Syntax::Nil,
    }
}

#[derive(Debug)]
pub struct NoWhitespace<'buf>(View<'buf>);
impl<'buf> ErrorMessage for NoWhitespace<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "expected whitespace")
    }
}

#[derive(Debug)]
pub struct NoBool<'buf>(View<'buf>);
impl<'buf> ErrorMessage for NoBool<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "expected boolean (#t or #f)")
    }
}

#[derive(Debug)]
pub struct NoNumber<'buf>(View<'buf>);
impl<'buf> ErrorMessage for NoNumber<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "expected number")
    }
}

#[derive(Debug)]
pub struct NoSymbol<'buf>(View<'buf>);
impl<'buf> ErrorMessage for NoSymbol<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "expected symbol")
    }
}

#[derive(Debug)]
pub struct NoSExpr<'buf>(View<'buf>);
impl<'buf> ErrorMessage for NoSExpr<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "expected '('")
    }
}

#[derive(Debug)]
pub struct UnclosedSExpr<'buf>(View<'buf>);
impl<'buf> ErrorMessage for UnclosedSExpr<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "missing ')', unclosed s expression")
    }
}

#[derive(Debug)]
pub struct UnexpectedToken<'buf>(View<'buf>);
impl<'buf> ErrorMessage for UnexpectedToken<'buf> {
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display(f)?;
        write!(f, "unexpected token")
    }
}
