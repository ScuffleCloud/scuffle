use scuffle_bytes_util::zero_copy::{I24Be, I48Be, U24Be, U48Be};
use scuffle_bytes_util::{BytesCow, StringCow};

pub trait IsoSized {
    fn size(&self) -> usize;
}

macro_rules! impl_sized {
    ($($t:ty),+) => {
        $(
            impl IsoSized for $t {
                fn size(&self) -> usize {
                    std::mem::size_of::<$t>()
                }
            }
        )+
    };
}

impl_sized!(f32, f64, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, char, ());

impl<T, const LEN: usize> IsoSized for [T; LEN]
where
    T: IsoSized,
{
    fn size(&self) -> usize {
        self.iter().map(|item| item.size()).sum()
    }
}

impl<T> IsoSized for Vec<T>
where
    T: IsoSized,
{
    fn size(&self) -> usize {
        self.iter().map(|item| item.size()).sum()
    }
}

impl<T> IsoSized for Option<T>
where
    T: IsoSized,
{
    fn size(&self) -> usize {
        match self {
            Some(item) => item.size(),
            None => 0,
        }
    }
}

impl IsoSized for BytesCow<'_> {
    fn size(&self) -> usize {
        self.len()
    }
}

impl IsoSized for StringCow<'_> {
    fn size(&self) -> usize {
        self.len()
    }
}

impl IsoSized for I24Be {
    fn size(&self) -> usize {
        3
    }
}

impl IsoSized for I48Be {
    fn size(&self) -> usize {
        6
    }
}

impl IsoSized for U24Be {
    fn size(&self) -> usize {
        3
    }
}

impl IsoSized for U48Be {
    fn size(&self) -> usize {
        6
    }
}
