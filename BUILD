load("@rules_rust//crate_universe:defs.bzl", "crates_vendor", "crate")

exports_files(["Cargo.toml"])

genrule(
    name = "cargo_metadata",
    outs = ["cargo-metadata.json"],
    srcs = glob([
        "**/Cargo.toml",
    ]) + ["Cargo.lock"],
    cmd = "cargo metadata --format-version 1 --manifest-path $(location //:Cargo.toml) > $@",
    visibility = ["//visibility:public"],
    tags = ["manual", "no-sandbox"],
)

crates_vendor(
    name = "crates_vendor",
    cargo_lockfile = "//:Cargo.lock",
    generate_build_scripts = True,
    manifests = [
        "//:Cargo.toml",
        "//cloud/core:Cargo.toml",
        "//cloud/proto:Cargo.toml",
        "//crates/aac:Cargo.toml",
        "//crates/amf0:Cargo.toml",
        "//crates/av1:Cargo.toml",
        "//crates/batching:Cargo.toml",
        "//crates/bootstrap:Cargo.toml",
        "//crates/bootstrap/derive:Cargo.toml",
        "//crates/bootstrap-telemetry:Cargo.toml",
        "//crates/bytes-util:Cargo.toml",
        "//crates/changelog:Cargo.toml",
        "//crates/context:Cargo.toml",
        "//crates/expgolomb:Cargo.toml",
        "//crates/ffmpeg:Cargo.toml",
        "//crates/flv:Cargo.toml",
        "//crates/future-ext:Cargo.toml",
        "//crates/h264:Cargo.toml",
        "//crates/h265:Cargo.toml",
        "//crates/http:Cargo.toml",
        "//crates/metrics:Cargo.toml",
        "//crates/metrics/derive:Cargo.toml",
        "//crates/mp4:Cargo.toml",
        "//crates/nutype-enum:Cargo.toml",
        "//crates/openapiv3_1:Cargo.toml",
        "//crates/postcompile:Cargo.toml",
        "//crates/pprof:Cargo.toml",
        "//crates/rtmp:Cargo.toml",
        "//crates/settings:Cargo.toml",
        "//crates/signal:Cargo.toml",
        "//crates/tinc:Cargo.toml",
        "//crates/tinc/build:Cargo.toml",
        "//crates/tinc/cel:Cargo.toml",
        "//crates/tinc/derive:Cargo.toml",
        "//crates/tinc/integration:Cargo.toml",
        "//crates/tinc/pb-prost:Cargo.toml",
        "//crates/transmuxer:Cargo.toml",
        "//dev-tools/xtask:Cargo.toml",
        "//dev-tools/test-runner:Cargo.toml",
    ],
    annotations = {
        "reqwest": [
            crate.annotation(
                rustc_flags = ["--cfg=reqwest_unstable"],
            ),
        ],
    },
    mode = "remote",
    supported_platform_triples = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
        "wasm32-unknown-unknown",
    ],
    vendor_path = "vendor",
    visibility = ["//visibility:public"],
    tags = ["manual"]
)
