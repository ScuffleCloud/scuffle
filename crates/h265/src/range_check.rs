macro_rules! range_check {
    ($n:expr,0, $upper:expr) => {{
        trait Unsigned {}
        impl Unsigned for u8 {}
        impl Unsigned for u16 {}
        impl Unsigned for u32 {}
        impl Unsigned for u64 {}
        impl Unsigned for usize {}
        impl Unsigned for ::std::num::NonZero<u8> {}
        impl Unsigned for ::std::num::NonZero<u16> {}
        impl Unsigned for ::std::num::NonZero<u32> {}
        impl Unsigned for ::std::num::NonZero<u64> {}
        impl Unsigned for ::std::num::NonZero<usize> {}

        #[inline(always)]
        const fn unsigned_type_check<N: Unsigned>(_: &N) {}
        unsigned_type_check(&$n);

        if $n > $upper {
            ::std::result::Result::Err(::std::io::Error::new(
                ::std::io::ErrorKind::InvalidData,
                format!("{} is out of range [0, {}]: {}", stringify!($n), $upper, $n),
            ))
        } else {
            ::std::result::Result::Ok(())
        }
    }};
    ($n:expr, $lower:expr, $upper:expr) => {{
        if $n < $lower || $n > $upper {
            ::std::result::Result::Err(::std::io::Error::new(
                ::std::io::ErrorKind::InvalidData,
                format!("{} is out of range [{}, {}]: {}", stringify!($n), $lower, $upper, $n),
            ))
        } else {
            ::std::result::Result::Ok(())
        }
    }};
}

pub(crate) use range_check;

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    #[test]
    fn u64() {
        let i = 2u64;
        range_check!(i, 0, 63).unwrap();
    }

    // Cannot be tested with postcompile because it's a private macro
    // #[test]
    // fn i64() {
    //     insta::assert_snapshot!(postcompile::compile! {
    //         use scuffle_h265::range_check;

    //         fn test() {
    //             let i = 2i64;
    //             range_check::range_check!(i, 0, 63);
    //         }
    //     });
    // }
}
