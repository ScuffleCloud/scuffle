use std::collections::BTreeSet;
use std::num::NonZeroU64;
use std::rc::Rc;

use redis_module_ext::utils::redis_time_millis;

pub type Str = Rc<str>;

pub fn now() -> NonZeroU64 {
    NonZeroU64::new(redis_time_millis() as u64).unwrap()
}

pub trait SetExt<T> {
    fn pop_first_if(&mut self, f: impl FnMut(&T) -> bool) -> Option<T>;
}

impl<T: Ord> SetExt<T> for BTreeSet<T> {
    fn pop_first_if(&mut self, mut f: impl FnMut(&T) -> bool) -> Option<T> {
        if f(self.first()?) { self.pop_first() } else { None }
    }
}
