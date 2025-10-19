use std::{
    cell::Cell,
    hint,
    marker::PhantomData,
    mem,
    process::abort,
    ptr::{self, NonNull},
    rc::Rc,
};

//#[repr(C)]
//struct Inner<T: ?Sized> {
//    count: Cell<usize>,
//    value: T,
//}
//
//impl <T: ?Sized> Inner<T> {
//    #[inline]
//    fn count(&self) -> usize {
//        self.count.get()
//    }
//    #[inline]
//    fn inc(&self) {
//        let count = self.count.get();
//        // We insert an `assume` here to hint LLVM at an otherwise
//        // missed optimization.
//        // SAFETY: The reference count will never be zero when this is
//        // called.
//        unsafe {
//            hint::assert_unchecked(count != 0);
//        }
//
//        let count = count.wrapping_add(1);
//        self.count.set(count);
//
//        // We want to abort on overflow instead of dropping the value.
//        // Checking for overflow after the store instead of before
//        // allows for slightly better code generation.
//        if count == 0 {
//            abort();
//        }
//    }
//    #[inline]
//    fn dec(&self) {
//        self.count.set(self.count() - 1);
//    }
//}
//
//pub struct Frc<T: ?Sized> {
//    ptr: NonNull<Inner<T>>,
//    phantom: PhantomData<Inner<T>>,
//}
//
//impl<T: ?Sized> Frc<T> {
//    #[inline]
//    unsafe fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
//        Self { ptr, phantom: PhantomData, }
//    }
//}
//
//impl <T: ?Sized> Frc<T> {
//    #[inline]
//    fn inner(&self) -> &Inner<T> {
//        unsafe { self.ptr.as_ref() }
//    }
//    #[inline]
//    pub fn ref_count(&self) -> usize {
//        self.inner().count.get()
//    }
//    // Non-inlined part of `drop`.
//    #[inline(never)]
//    unsafe fn drop_slow(&mut self) {
//        // Reconstruct the "strong weak" pointer and drop it when this
//        // variable goes out of scope. This ensures that the memory is
//        // deallocated even if the destructor of `T` panics.
//        let _ptr = self.ptr;
//
//        unsafe {
//            ptr::drop_in_place(&mut (*self.ptr.as_ptr()).value);
//        }
//    }
//}
//
//impl <T> Frc<T> {
//    #[inline]
//    pub fn cons(value: T) -> Self {
//        unsafe { Self::from_inner(Box::leak(Box::new(Inner { count: Cell::new(1), value })).into()) }
//    }
//}
//
//impl <T: ?Sized> Drop for Frc<T> {
//    #[inline]
//    fn drop(&mut self) {
//        unsafe {
//            self.inner().dec();
//            if self.inner().count() == 0 {
//                self.drop_slow();
//            }
//        }
//    }
//}
//
//impl <T: ?Sized> AsRef<T> for Frc<T> {
//    fn as_ref(&self) -> &T {
//        &self.inner().value
//    }
//}
//
//impl <T: Clone> Frc<T> {
//    #[inline]
//    pub fn decons(self) -> T {
//        if self.ref_count() != 1 {
//            self.inner().value.clone()
//        }
//        else {
//            unsafe {
//                self.ptr.read().value
//            }
//        }
//    }
//    #[inline]
//    pub fn map(mut self, f: impl FnOnce(T) -> T) -> Self {
//        if self.ref_count() != 1 {
//            Self::cons(f(self.inner().value.clone()))
//        }
//        else {
//            unsafe {
//                self.ptr.as_mut().value = f(self.ptr.read().value)
//            }
//            self
//        }
//    }
//}
//
//impl <T: ?Sized> Clone for Frc<T> {
//    #[inline]
//    fn clone(&self) -> Self {
//        unsafe {
//            self.inner().inc();
//            Self::from_inner(self.ptr)
//        }
//    }
//}

pub struct Frc<T: ?Sized> {
    inner: Rc<T>,
}

impl<T> Frc<T> {
    #[inline]
    pub fn cons(value: T) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }
}

impl<T: ?Sized> AsRef<T> for Frc<T> {
    fn as_ref(&self) -> &T {
        &self.inner.as_ref()
    }
}

impl<T: Clone> Frc<T> {
    #[inline]
    pub fn decons(self) -> T {
        Rc::unwrap_or_clone(self.inner)
    }
}
impl<T: Clone + Default> Frc<T> {
    #[inline]
    pub fn map(mut self, f: impl FnOnce(T) -> T) -> Self {
        let m = Rc::make_mut(&mut self.inner);
        *m = f(mem::take(m));
        self
    }
}

impl<T: ?Sized> Clone for Frc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
