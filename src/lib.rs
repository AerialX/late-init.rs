#![no_std]

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::{ptr, fmt};

#[repr(transparent)]
pub struct LateInitUnchecked<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Sync> Sync for LateInitUnchecked<T> { }
unsafe impl<T: Send> Send for LateInitUnchecked<T> { }

#[cfg(feature = "const-default")]
impl<T> const_default::ConstDefault for LateInitUnchecked<T> {
    const DEFAULT: Self = LateInitUnchecked { inner: UnsafeCell::new(MaybeUninit::uninit()) };
}

impl<T: fmt::Debug> fmt::Debug for LateInitUnchecked<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("LateInitUnchecked")
            .field(self.late_get_ref())
            .finish()
    }
}

impl<T> LateInitUnchecked<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[inline]
    pub const fn with(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::new(value)),
        }
    }

    #[inline]
    pub fn late_init_mut(&mut self, value: T) {
        unsafe {
            self.late_init(value)
        }
    }

    /// Repeated initializations will leak previous values without dropping them.
    #[inline(always)]
    pub unsafe fn late_init<I: Into<T>>(&self, value: I) {
        //*self.inner.get() = MaybeUninit::new(value);
        ptr::write((*self.inner.get()).as_mut_ptr(), value.into());
    }

    #[inline]
    pub fn late_ptr(&self) -> *const T {
        unsafe {
            (*self.inner.get()).as_ptr()
        }
    }

    #[inline]
    pub fn late_ptr_mut(&self) -> *mut T {
        unsafe {
            (*self.inner.get()).as_mut_ptr()
        }
    }

    #[inline]
    pub fn late_get_ref(&self) -> &T {
        unsafe {
            &*self.late_ptr()
        }
    }

    #[inline]
    pub fn late_get_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.late_ptr_mut()
        }
    }

    #[inline]
    pub unsafe fn late_get_mut_unchecked(&self) -> &mut T {
        &mut *self.late_ptr_mut()
    }
}

impl<T> Deref for LateInitUnchecked<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.late_get_ref()
    }
}

impl<T> DerefMut for LateInitUnchecked<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.late_get_mut()
    }
}

#[repr(transparent)]
pub struct LateInit<T> {
    inner: UnsafeCell<Option<T>>,
}

unsafe impl<T: Sync> Sync for LateInit<T> { }
unsafe impl<T: Send> Send for LateInit<T> { }

#[cfg(feature = "const-default")]
impl<T> const_default::ConstDefault for LateInit<T> {
    const DEFAULT: Self = LateInit { inner: UnsafeCell::new(None) };
}

impl<T: fmt::Debug> fmt::Debug for LateInit<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut fmt = fmt.debug_tuple("LateInit");
        match self.late_try_get_ref() {
            Some(inner) => { fmt.field(inner); },
            None => { fmt.field(&"<UNINIT>"); },
        }
        fmt.finish()
    }
}

impl<T> LateInit<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    #[inline]
    pub const fn with(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(Some(value)),
        }
    }

    #[inline]
    pub fn late_init_mut(&mut self, value: T) {
        unsafe {
            self.late_init(value)
        }
    }

    #[inline]
    pub unsafe fn late_init<I: Into<T>>(&self, value: I) {
        let inner = self.late_inner_mut();
        debug_assert!(inner.is_none());
        *inner = Some(value.into());
    }

    #[inline]
    unsafe fn late_inner_mut(&self) -> &mut Option<T> {
        &mut *self.inner.get()
    }

    #[inline]
    fn late_inner(&self) -> &Option<T> {
        unsafe {
            &*self.inner.get()
        }
    }

    #[inline]
    fn late_unexpected() -> ! {
        // TODO: feature to control unreachableness
        debug_assert!(false);
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }

    pub fn late_ptr(&self) -> *const T {
        self.late_inner().as_ref().map(|inner| inner as *const _).unwrap_or(ptr::null())
    }

    #[inline]
    pub fn late_ptr_mut(&self) -> *mut T {
        self.late_ptr() as *mut _
    }

    #[inline]
    pub fn late_try_get_ref(&self) -> Option<&T> {
        self.late_inner().as_ref()
    }

    #[inline]
    pub fn late_try_get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.late_inner_mut().as_mut()
        }
    }

    pub fn late_get_ref(&self) -> &T {
        match self.late_try_get_ref() {
            Some(inner) => inner,
            None => Self::late_unexpected(),
        }
    }

    pub fn late_get_mut(&mut self) -> &mut T {
        match self.late_try_get_mut() {
            Some(inner) => inner,
            None => Self::late_unexpected(),
        }
    }

    pub unsafe fn late_get_mut_unchecked(&self) -> &mut T {
        match self.late_inner_mut().as_mut() {
            Some(inner) => inner,
            None => Self::late_unexpected(),
        }
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.late_get_ref()
    }
}

impl<T> DerefMut for LateInit<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.late_get_mut()
    }
}
