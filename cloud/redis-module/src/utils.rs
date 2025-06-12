use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::rc::Rc;

pub type Str = Rc<str>;

macro_rules! extern_fn {
    (
        |$($arg:ident:$arg_ty:ty),*$(,)?| $body:expr
    ) => {{
        const fn cast<R>(func: fn($($arg_ty),*) -> R) -> unsafe extern "C" fn($($arg_ty),*) -> R {
            unsafe { std::mem::transmute(func) }
        }

        cast(|$($arg:$arg_ty),*| $body)
    }};
}

pub(super) use extern_fn;

pub fn now() -> NonZeroU64 {
    NonZeroU64::new(chrono::Utc::now().timestamp() as u64).unwrap()
}

pub trait SetExt<T> {
    fn pop_first_if(&mut self, f: impl FnMut(&T) -> bool) -> Option<T>;
}

impl<T: Ord> SetExt<T> for BTreeSet<T> {
    fn pop_first_if(&mut self, mut f: impl FnMut(&T) -> bool) -> Option<T> {
        if f(self.first()?) { self.pop_first() } else { None }
    }
}
