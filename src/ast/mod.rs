use core::{error, fmt};

use crate::peg::{self, PEGParser, PEGParserExt, graphemes::GraphemePEGParser};

#[derive(Debug)]
pub enum ASTError {
    NoAtom,
    NoSExpr,
    NoExpr,
    UnclosedSExpr,
}

impl fmt::Display for ASTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl error::Error for ASTError {}

#[derive(Debug)]
pub enum ASTNode<'buf> {
    SExpr(Vec<ASTNode<'buf>>),
    Atom(&'buf str),
}

impl<'buf> fmt::Display for ASTNode<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(str) => write!(f, "{}", str),
            Self::SExpr(nodes) => {
                if nodes.is_empty() {
                    write!(f, "()")?;
                    return Ok(());
                }
                write!(f, "(")?;
                nodes.iter().enumerate().try_for_each(|(i, node)| {
                    if i == nodes.len() - 1 {
                        write!(f, "{})", node.to_string().replace("\n", "\n  "))
                    } else {
                        write!(f, "{}\n  ", node.to_string().replace("\n", "\n  "))
                    }
                })?;
                Ok(())
            }
        }
    }
}

const ATOM_ILLEGALS: &[&str] = &[" ", "\n", "\t", "(", ")", ";"];

#[inline(always)]
fn atom_parser<'buf>() -> impl peg::PEGParser<&'buf str, ASTNode<'buf>, ASTError> {
    peg::graphemes::capture_while(|cap| !ATOM_ILLEGALS.contains(&cap.grapheme))
        .map_err(|_| ASTError::NoAtom)
        .and_then(|res| match res {
            "" => Err(ASTError::NoAtom),
            x => Ok(ASTNode::Atom(x)),
        })
}

#[inline(always)]
fn whitespace<'buf>() -> impl peg::PEGParser<&'buf str, (), ()> {
    peg::graphemes::capture_while(|cap| [" ", "\n", "\t"].contains(&cap.grapheme))
        .map_err(|_| ())
        .and_then(|res| match res {
            "" => Err(()),
            _ => Ok(()),
        })
}

fn s_expr_parser<'buf>() -> impl peg::PEGParser<&'buf str, ASTNode<'buf>, ASTError> {
    "(".map_err(|_| ASTError::NoSExpr)
        .then(whitespace().optional())
        .then_right(expr_parser().then_left(whitespace().optional()).greedy())
        .flatmap(move |res| {
            move |buf| match res {
                Ok((exprs, peg::Either::Left(err))) => match err {
                    ASTError::NoExpr => Ok((buf, ASTNode::SExpr(exprs))),
                    err => Err(err),
                },
                Err(peg::Either::Left(peg::Either::Left(err))) => Err(err),
            }
        })
        .then_left(")".map_err(|_| ASTError::UnclosedSExpr))
        .map_err(peg::Either::fuse)
}

#[inline(always)]
fn expr_parser<'buf>() -> impl peg::PEGParser<&'buf str, ASTNode<'buf>, ASTError> {
    // lazily recursively call s-expr (pray we don't run out of stack memory :skull:)
    let rec = |buf| s_expr_parser().parse(buf);
    // parse either the atom or the s-expr
    atom_parser()
        .or(rec)
        .map(peg::Either::fuse)
        .map_err(|err| match err {
            (_, ASTError::UnclosedSExpr) => ASTError::UnclosedSExpr,
            (ASTError::NoAtom, ASTError::NoSExpr) => ASTError::NoExpr,
            (ASTError::NoAtom, err) => err,
            (err, ASTError::NoSExpr) => err,
            (_, err) => err,
        })
}

#[inline(always)]
pub fn atlas_parser<'buf>() -> impl peg::PEGParser<&'buf str, Vec<ASTNode<'buf>>, ASTError> {
    expr_parser()
        .then_left(whitespace().optional())
        .map_err(|peg::Either::Left(err)| err)
        .greedy()
        .map_err(|_| unreachable!())
        .and_then(|(res, err)| match err {
            ASTError::NoExpr => Ok(res),
            _ => Err(err),
        })
        .expect_end(|| ASTError::NoExpr)
}
