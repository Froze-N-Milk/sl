use core::fmt;
use std::rc::Rc;

use crate::fastpass::{
    self, CaptureWhile, Either, ErrorMessage, Infallible, ParseResult, Parser, View,
};
use crate::interpreter::Value;

const SYMBOL_ILLEGALS: &[char] = &[' ', '\r', '\n', '\t', '(', ')', ';'];

#[inline(always)]
fn symbol<'buf>(buf: View<'buf>) -> ParseResult<'buf, Value<'buf>, NoSymbol<'buf>> {
    let Ok((buf, res)) =
        fastpass::CaptureWhile(|_, char| !SYMBOL_ILLEGALS.contains(&char)).parse(buf);
    match res {
        "" => Err(NoSymbol(buf)),
        x => Ok((buf, Value::Symbol(x))),
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
        Value::Symbol(res) => assert_eq!("abc", res),
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

#[inline(always)]
fn bool<'buf>(buf: View<'buf>) -> ParseResult<'buf, Value<'buf>, NoBool<'buf>> {
    match "#t".or("#f").parse(buf) {
        Ok((buf, Either::L(_))) => Ok((buf, Value::Bool(true))),
        Ok((buf, Either::R(_))) => Ok((buf, Value::Bool(false))),
        Err(_) => Err(NoBool(buf)),
    }
}

#[inline(always)]
fn sexpr<'buf>(
    buf: View<'buf>,
) -> ParseResult<'buf, Value<'buf>, Either<NoSExpr<'buf>, UnclosedSExpr<'buf>>> {
    let open = "(".map_err(|(buf, _, _)| Err(Either::L(NoSExpr(buf))));
    let close = ")".map_err(|(buf, _, _)| Err(Either::R(UnclosedSExpr(buf))));

    let (buf, _) = open.parse(buf)?;

    let Ok((buf, (exprs, Either::L(err)))) = expr.then_left(swallow).greedy().parse(buf);

    let exprs = match err {
        unclosed_sexpr @ Either::R(_) => return Err(unclosed_sexpr),
        _ => Value::List(Rc::from(exprs)),
    };

    let (buf, _) = close.parse(buf)?;
    let Ok((buf, _)) = swallow.parse(buf);
    Ok((buf, exprs))
}

#[inline(always)]
fn expr<'buf>(
    buf: View<'buf>,
) -> ParseResult<'buf, Value<'buf>, Either<NoSExpr<'buf>, UnclosedSExpr<'buf>>> {
    match bool.then_left(swallow).parse(buf) {
        Ok(res) => return Ok(res),
        _ => (),
    };
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
) -> Result<Value<'buf>, Either<UnclosedSExpr<'buf>, UnexpectedToken<'buf>>> {
    let Ok((buf, _)) = swallow.parse(buf);

    let Ok((buf, (exprs, err))) = expr.greedy().parse(buf);
    let exprs = match err {
        Either::R(err) => return Err(Either::L(err)),
        _ => Value::List(Rc::from(exprs)),
    };

    match buf.as_str() {
        "" => Ok(exprs),
        _ => Err(Either::R(UnexpectedToken(buf))),
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
