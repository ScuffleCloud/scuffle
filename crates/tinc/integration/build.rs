#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

fn main() {
    tinc_build::Config::prost()
        .btree_map(".")
        .compile_protos(
            &[
                "pb/simple.proto",
                "pb/recursive.proto",
                "pb/simple_enum.proto",
                "pb/nested.proto",
                "pb/flattened.proto",
                "pb/oneof.proto",
                "pb/renamed.proto",
                "pb/visibility.proto",
                "pb/well_known.proto",
                "pb/simple_service.proto",
            ],
            &["pb"],
        )
        .unwrap();
}
