//! A helper macro to ensure that a number is within the specified [$lower, $upper] bounds.

/// Enforces that a number is within the specified \[LOWER, UPPER\] bounds.
///
/// The brackets indicate that this range is inclusive on both sides.
#[macro_export]
macro_rules! range_check {
    ($n:expr, $lower:expr, $upper:expr) => {{
        let n = $n;

        #[allow(unused_comparisons, clippy::manual_range_contains)]
        if n < $lower || n > $upper {
            ::std::result::Result::Err(::std::io::Error::new(
                ::std::io::ErrorKind::InvalidData,
                format!("{} is out of range [{}, {}]: {}", stringify!($n), $lower, $upper, n),
            ))
        } else {
            ::std::result::Result::Ok(())
        }
    }};
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    #[test]
    fn u64() {
        let i = 2u64;
        range_check!(i, 0, 63).unwrap();
    }
}
