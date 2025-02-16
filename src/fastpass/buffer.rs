pub trait Buffer : Sized + Copy {}

pub struct SliceBuffer<'buf, T> {
    buffer: &'buf [T],
    start: usize,
    len: usize,
}

impl <'buf, T> core::ops::Index<usize> for SliceBuffer<'buf, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl <'buf, T> Clone for SliceBuffer<'buf, T> {
    fn clone(&self) -> Self {
        Self {
            ..*self
        }
    }
}

impl <'buf, T> Copy for SliceBuffer<'buf, T> {}

impl <'buf, T> Buffer for SliceBuffer<'buf, T> {}
