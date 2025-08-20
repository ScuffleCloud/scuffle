#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

fn main() {
    let mut config = tinc_build::Config::prost();
    config
        .btree_map(".")
        .float_with_non_finite_vals(".floats.FloatMessageWithNonFinite")
        .float_with_non_finite_vals(".floats.FloatMessageWithSomeNonFinite.f32_with_non_finite_serializer")
        .float_with_non_finite_vals(".floats.FloatMessageWithSomeNonFinite.f64_with_non_finite_serializer")
        .float_with_non_finite_vals(".expressions.FloatExpressions")
        .float_with_non_finite_vals(".expressions.DoubleExpressions");

    if let Some(pre_compiled_fds) = std::env::var_os("TINC_INTEGRATION_COMPILED_FD") {
        let fds = std::fs::read(pre_compiled_fds).expect("pre_compiled_fds not found");
        config.load_fds(fds.as_slice()).unwrap();
    } else {
        config
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
                    "pb/bytes_service.proto",
                    "pb/expressions.proto",
                    "pb/floats.proto",
                ],
                &["pb"],
            )
            .unwrap();
    }
}
