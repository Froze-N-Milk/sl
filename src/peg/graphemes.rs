use unicode_segmentation::UnicodeSegmentation;

use crate::peg;

impl<'buf> peg::PEGParser<&'buf str, &'buf str, ()> for &'buf str {
    fn parse(&self, buf: &'buf str) -> Result<(&'buf str, &'buf str), ()> {
        if buf.len() < self.len() {
            return Err(());
        }
        if buf[..self.len()] == **self {
            return Ok((&buf[self.len()..], self));
        }

        Err(())
    }
}

pub struct Capture<'buf> {
    /// total captured string so far
    pub total: &'buf str,
    /// most recently processed grapheme
    pub grapheme: &'buf str,
}

/// calls [f] as long as it continues to return `true`
///
/// each call will be given a [Capture] containing the total captured string so far,
/// along with the last captured grapheme.
pub fn capture_while<'buf, F: Fn(Capture<'buf>) -> bool>(
    f: F,
) -> impl peg::PEGParser<&'buf str, &'buf str, peg::Infallible> {
    move |buf: &'buf str| {
        let mut graphemes = buf.grapheme_indices(true);
        let mut bisect = 0;
        loop {
            let Some((offset, grapheme)) = graphemes.next() else {
                return Ok(("", buf));
            };
            let i = offset + grapheme.len();
            if !f(Capture {
                total: &buf[..i],
                grapheme,
            }) {
                return Ok((&buf[bisect..], &buf[..bisect]));
            }
            bisect = i;
        }
    }
}

pub trait GraphemePEGParser<'buf, T, E>: peg::PEGParser<&'buf str, T, E> + Sized {
    fn expect_end(self, f: impl Fn() -> E) -> impl peg::PEGParser<&'buf str, T, E> {
        move |buf| -> Result<(&'buf str, T), E> {
            let res = self.parse(buf)?;
            if res.0 == "" {
                Ok(res)
            } else {
                Err(f())
            }
        }
    }
}

impl<'buf, T, E, PEG> GraphemePEGParser<'buf, T, E> for PEG where
    PEG: peg::PEGParser<&'buf str, T, E>
{
}
