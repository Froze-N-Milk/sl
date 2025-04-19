use core::fmt;

pub trait ErrorMessage: Sized {
	fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<'a> ErrorMessage for &'a str {
	fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)
	}
}

#[derive(Debug)]
pub enum Infallible {}
impl ErrorMessage for Infallible {
	fn display(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
		unreachable!()
	}
}

pub struct Display<E: ErrorMessage>(pub E);
impl<E: ErrorMessage> fmt::Display for Display<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.display(f)
	}
}

macro_rules! impl_errormessage_for_tuple {
    ($($errormessage:ident),+) => (
        #[allow(non_snake_case)]
        impl<$($errormessage),+> ErrorMessage for ($($errormessage),+,)
where
    $($errormessage: ErrorMessage),+
{
    #[inline(always)]
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ($(ref $errormessage),+,) = *self;
        $($errormessage.display(f)?;)+
            Ok(())
    }
}
    )
}

macro_rules! impl_errormessage_for_tuples {
    ($errormessage1:ident, $($errormessage:ident),+) => {
        impl_errormessage_for_tuples!(__impl $errormessage1; $($errormessage),+);
    };
    (__impl $($errormessage:ident),+; $errormessage1:ident $(,$errormessage2:ident)*) => {
        impl_errormessage_for_tuple!($($errormessage),+);
        impl_errormessage_for_tuples!(__impl $($errormessage),+, $errormessage1; $($errormessage2),*);
    };
    (__impl $($errormessage:ident),+;) => {
        impl_errormessage_for_tuple!($($errormessage),+);
    }
}

impl_errormessage_for_tuples!(
	EM1, EM2, EM3, EM4, EM5, EM6, EM7, EM8, EM9, EM10, EM11, EM12, EM13, EM14, EM15, EM16,
	EM17, EM18, EM19, EM20, EM21
);
