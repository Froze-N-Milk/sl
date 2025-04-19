use core::fmt;
use std::rc::Rc;

use crate::fastpass::{self, CaptureWhile, Either, ErrorMessage, Infallible, Parser, View};

const SYMBOL_ILLEGALS: &[char] = &[' ', '\r', '\n', '\t', '(', ')', ';'];

#[inline(always)]
fn symbol_parser<'buf>() -> impl Parser<'buf, Output = Expression<'buf>, Error = NoSymbol<'buf>> {
	fastpass::CaptureWhile(|_, char| !SYMBOL_ILLEGALS.contains(&char)).flatmap(|Ok(res)| {
		move |buf| match res {
			"" => Err(NoSymbol(buf)),
			x => Ok((buf, Expression::Symbol(x))),
		}
	})
}

#[test]
fn symbol_test() {
	let parser = symbol_parser();
	let buf = View::new("abc");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, res) = res.unwrap();
	assert_eq!("", buf.as_str());
	match res {
		Expression::SExpr(_) => panic!(),
		Expression::Symbol(res) => assert_eq!("abc", res),
	}
}

#[inline(always)]
fn comment<'buf>() -> impl Parser<'buf, Error = Infallible> {
	";".then(CaptureWhile(|_, c| c != '\n')).map(|_| Ok(()))
}

#[test]
fn comment_test() {
	let parser = comment();
	let buf = View::new(";    ");
	let res = Parser::parse(&parser, buf);
	assert!(res.is_ok());
	let (buf, _) = res.unwrap();
	assert_eq!("", buf.as_str());
}

#[inline(always)]
fn whitespace<'buf>() -> impl Parser<'buf> {
	fastpass::CaptureWhile(|_, char| [' ', '\r', '\n', '\t'].contains(&char)).map(|Ok(res)| {
		match res {
			"" => Err("expected whitespace"),
			_ => Ok(()),
		}
	})
}

#[inline(always)]
fn swallow<'buf>() -> impl Parser<'buf, Output = (), Error = Infallible> {
	let p = whitespace().or(comment());
	move |mut buf| loop {
		let Ok((buf2, _)) = p.parse(buf);
		if buf.as_str() == buf2.as_str() {
			return Ok((buf2, ()));
		}
		buf = buf2;
	}
}

#[inline(always)]
fn s_expr_parser<'buf>(
) -> impl Parser<'buf, Output = Expression<'buf>, Error = Either<NoSExpr<'buf>, UnclosedSExpr<'buf>>> {
	let open = "(".map_err(|(buf, _, _)| Err(NoSExpr(buf)));
	let close = ")".map_err(|(buf, _, _)| Err(UnclosedSExpr(buf)));

	open.then(swallow())
		.then_right(expr_parser().then_left(swallow()).greedy())
		.map(|res| match res {
			//Ok((_, Either::L(Either::R((_, Either::R(err)))))) => Err(Either::R(err)),
			//Ok((exprs, _)) => Ok(ASTNode::SExpr(exprs)),
			Ok((exprs, Either::L((_, Either::L(_))))) => Ok(Expression::SExpr(Rc::from(exprs))),
			Ok((_, Either::L((_, Either::R(err))))) => Err(Either::R(err)),
			Err(Either::L(Either::L(err))) => Err(Either::L(err)),
		})
		.then_left(close)
		.then_left(swallow())
		.map_err(|Either::L(err)| match err {
			Either::L(err) => Err(err),
			Either::R(err) => Err(Either::R(err)),
		})
}

#[inline(always)]
fn expr_parser<'buf>() -> impl Parser<
	'buf,
	Output = Expression<'buf>,
	Error = (NoSymbol<'buf>, Either<NoSExpr<'buf>, UnclosedSExpr<'buf>>),
> {
	// lazily recursively call s-expr (pray we don't run out of stack memory :skull:)
	let rec = |buf| s_expr_parser().parse(buf);
	// parse either the symbol or the s-expr
	symbol_parser()
		.or(rec)
		.map_ok(|res| Ok(res.fuse()))
		.then_left(swallow())
		.map_err(|Either::L(err)| Err(err))
}

#[inline(always)]
pub fn sl_parser<'buf>() -> impl Parser<
	'buf,
	Output = Vec<Expression<'buf>>,
	Error = Either<UnclosedSExpr<'buf>, UnexpectedToken<'buf>>,
> {
	swallow()
		.then_right(expr_parser().greedy())
		.map(|Ok((exprs, (_, err)))| match err {
			Either::L(_) => Ok(exprs),
			Either::R(err) => Err(err),
		})
		.expect(|buf| match buf.as_str() {
			"" => None,
			_ => Some(UnexpectedToken(buf)),
		})
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

#[derive(Debug)]
pub enum Expression<'buf> {
	SExpr(Rc<[Expression<'buf>]>),
	Symbol(&'buf str),
}

impl <'env> Clone for Expression<'env> {
    fn clone(&self) -> Self {
        match self {
            Self::SExpr(es) => Self::SExpr(es.clone()),
            Self::Symbol(s) => Self::Symbol(s),
        }
    }
}

impl<'buf> fmt::Display for Expression<'buf> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Symbol(str) => write!(f, "{}", str),
			Self::SExpr(nodes) => {
				if nodes.is_empty() {
					write!(f, "()")?;
					return Ok(());
				}
				write!(f, "(")?;
				nodes.iter().enumerate().try_for_each(|(i, node)| {
					if i == nodes.len() - 1 {
						write!(
							f,
							"{})",
							node.to_string().replace("\n", "\n  ")
						)
					} else {
						write!(
							f,
							"{}\n  ",
							node.to_string().replace("\n", "\n  ")
						)
					}
				})?;
				Ok(())
			}
		}
	}
}
