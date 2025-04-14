#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

fn main() {
    let config = tinc_build::Config::new();

    let mut prost_config = prost_build::Config::new();

    prost_config.btree_map(["."]);

    config
        .with_prost(prost_config)
        .compile_protos(
            &[
                "pb/simple.proto",
                "pb/recursive.proto",
                "pb/simple_enum.proto",
                "pb/nested.proto",
                "pb/flattened.proto",
                "pb/oneof.proto",
            ],
            &["pb"],
        )
        .unwrap();
}
