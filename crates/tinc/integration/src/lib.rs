#![cfg(test)]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

mod flattened;
mod nested;
mod oneof;
mod recursive;
mod simple;
mod simple_enum;
