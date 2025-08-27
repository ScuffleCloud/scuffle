#![cfg(test)]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

mod bytes_service;
mod expressions;
mod flattened;
mod floats;
mod nested;
mod oneof;
mod recursive;
mod renamed;
mod simple;
mod simple_enum;
mod simple_service;
mod visibility;
mod well_known;
