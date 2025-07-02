load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_vendor")
load("@bazel_skylib//rules:common_settings.bzl", "string_flag")

exports_files(["Cargo.toml", ".config/nextest.toml"])

string_flag(
    name = "test_profile",
    build_setting_default = "default",
    values = ["ci", "default"],
    visibility = ["//visibility:public"],
)

string_flag(
    name = "test_workspace_root",
    build_setting_default = "",
    visibility = ["//visibility:public"],
)

crates_vendor(
    name = "crates_vendor",
    annotations = {
        "reqwest": [
            crate.annotation(
                rustc_flags = ["--cfg=reqwest_unstable"],
            ),
        ],
        "rusty_ffmpeg": [
            crate.annotation(
                build_script_env = {
                    "CLANG_PATH": "$${pwd}/$(CC)",
                    "FFMPEG_INCLUDE_DIR": "$(execpath @//bazel/toolchains/ffmpeg:include)",
                    "FFMPEG_DLL_PATH": "$(execpath @//bazel/toolchains/ffmpeg:lib)",
                    "LIBCLANG_PATH": "$(execpath @//bazel/toolchains/llvm:libclang)",
                    "BINDGEN_EXTRA_CLANG_ARGS": "--sysroot=$(execpath @//bazel/toolchains/llvm:sysroot)",
                },
                build_script_data = [
                    "@//bazel/toolchains/ffmpeg:lib",
                    "@//bazel/toolchains/ffmpeg:include",
                    "@//bazel/toolchains/llvm:libclang",
                    "@//bazel/toolchains/llvm:sysroot",
                ],
            ),
        ],
    },
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
    tags = ["manual"],
    vendor_path = "vendor",
    visibility = ["//visibility:public"],
)
