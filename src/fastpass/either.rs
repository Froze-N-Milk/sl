use super::ErrorMessage;

pub enum Either<L, R> {
	L(L),
	R(R),
}

impl<T> Either<T, T> {
	pub fn fuse(self) -> T {
		match self {
			Either::L(t) => t,
			Either::R(t) => t,
		}
	}
}

impl<L: ErrorMessage, R: ErrorMessage> ErrorMessage for Either<L, R> {
	fn display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Either::L(l) => l.display(f),
			Either::R(r) => r.display(f),
		}
	}
}
