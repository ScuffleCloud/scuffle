load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_vendor", "render_config")
load("//:cargo_targets.bzl", "cargo_manifests")

crates_vendor(
    name = "cargo_vendor",
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
                    "BINDGEN_EXTRA_CLANG_ARGS": "$(CFLAGS)",
                    "FFMPEG_DLL_PATH": "$(execpath @//build/toolchains/ffmpeg:lib)",
                    "FFMPEG_INCLUDE_DIR": "$(execpath @//build/toolchains/ffmpeg:include)",
                    "LIBCLANG_PATH": "$(execpath @//build/toolchains/clang:libclang)",
                },
                build_script_data = [
                    "@//build/toolchains/ffmpeg:lib",
                    "@//build/toolchains/ffmpeg:include",
                    "@//build/toolchains/clang:libclang",
                ],
            ),
        ],
        "aws-lc-sys": [
            crate.annotation(
                build_script_env = {
                    # aws-lc-sys does not respect `LDFLAGS` so we need to
                    # add `-fuse-ld=lld` to instruct the compiler to use
                    # `lld` instead of `ld. We also need to add `-Wno-error`
                    # since this crate has a lot of warnings and our typical
                    # toolchain disallows them and then we silence them with `-w`.
                    # See https://github.com/aws/aws-lc-rs/issues/858
                    "CFLAGS": "$(CFLAGS) -fuse-ld=lld -Wno-error -w",
                },
            ),
        ],
    },
    render_config = render_config(
        default_alias_rule = "opt",
    ),
    cargo_lockfile = "//:Cargo.lock",
    generate_build_scripts = True,
    manifests = cargo_manifests(),
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
    vendor_path = "cargo",
    visibility = ["//visibility:public"],
)

crates_vendor(
    name = "bindeps",
    cargo_lockfile = ":Cargo.bindeps.lock",
    generate_build_scripts = True,
    packages = {
        "cargo-deny": crate.spec(
            artifact = "bin",
            version = "0.18.3",
        ),
        "just": crate.spec(
            artifact = "bin",
            version = "1.42.4",
        ),
        "miniserve": crate.spec(
            artifact = "bin",
            version = "0.31.0",
        ),
        "rust-analyzer": crate.spec(
            artifact = "bin",
            git = "https://github.com/TroyKomodo/rust-analyzer.git",
            rev = "e9ca9f8ec88a207891c32da4fc6523026ea03c6f",
        ),
    },
    cargo_config = ":rust.bindeps.cargo.toml",
    mode = "remote",
    supported_platform_triples = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
    ],
    generate_binaries = True,
    stable_as_nightly = True,
    tags = ["manual"],
    vendor_path = "bindeps",
    visibility = ["//visibility:public"],
)
