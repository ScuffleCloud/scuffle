#[inline]
pub const fn len_sum(to_concat: &'static [&'static [&'static str]]) -> usize {
    let mut len = 0;
    let mut i = 0;
    while i < to_concat.len() {
        len += to_concat[i].len();
        i += 1;
    }
    len
}

#[inline]
pub const fn concat_array<const LEN: usize>(to_concat: &'static [&'static [&'static str]]) -> [&'static str; LEN] {
    let mut res: [&'static str; LEN] = [""; LEN];
    let mut shift = 0;
    let mut i = 0;
    while i < to_concat.len() {
        let to_concat_one = to_concat[i];
        let mut j = 0;
        while j < to_concat_one.len() {
            res[j + shift] = to_concat_one[j];
            j += 1;
        }
        shift += j;
        i += 1;
    }
    res
}

#[macro_export]
#[doc(hidden)]
macro_rules! __private_const_concat_str_array {
    ($($rest:expr),*) => {{
        const TO_CONCAT: &[&[&str]] = &[$($rest),*];
        const LEN: usize = $crate::__private::const_macros::len_sum(TO_CONCAT);
        &$crate::__private::const_macros::concat_array::<LEN>(TO_CONCAT)
    }};
    ($($rest:expr),*,) => {
        $crate::__private_const_concat_str_array!($($rest),*)
    };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_len_sum() {
        const TO_CONCAT: &[&[&str]] = &[&["a", "b"], &["c"]];
        assert_eq!(len_sum(TO_CONCAT), 3);
    }

    #[test]
    fn test_concat_array() {
        const TO_CONCAT: &[&[&str]] = &[&["a", "b"], &["c"]];
        const LEN: usize = len_sum(TO_CONCAT);
        let result = concat_array::<LEN>(TO_CONCAT);
        assert_eq!(result, ["a", "b", "c"]);
    }
}
