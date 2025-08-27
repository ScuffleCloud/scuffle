load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_vendor", "render_config")
load("//:cargo_targets.bzl", "cargo_manifests")

crates_vendor(
    name = "cargo_vendor",
    cargo_bazel = "//build/toolchains/rust:cargo-bazel",
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
    cargo_bazel = "//build/toolchains/rust:cargo-bazel",
    cargo_lockfile = ":Cargo.bindeps.lock",
    generate_build_scripts = True,
    annotations = {
        binary: [crate.annotation(gen_binaries = True)]
        for binary in ["cargo-deny", "cargo-insta", "just", "miniserve", "rust-analyzer", "mergiraf"]
    } | {
        "dprint": [
            crate.annotation(gen_binaries = True, gen_build_script = False, rustc_env = {
                "TARGET": "",
                "RUSTC_VERSION_TEXT": "",
            })
        ],
        "bzip2-sys": [
            crate.annotation(
                build_script_env = {
                    "CFLAGS": "$(CFLAGS) -Wno-error -w",
                },
            ),
        ],
    },
    packages = {
        "cargo-deny": crate.spec(
            artifact = "bin",
            version = "0.18.3",
        ),
        "cargo-insta": crate.spec(
            artifact = "bin",
            version = "1.43.1",
        ),
        # Currently using a fork due to https://github.com/dprint/dprint/issues/1020
        "dprint": crate.spec(
            artifact = "bin",
            git = "https://github.com/TroyKomodo/dprint.git",
            rev = "a966de1d32d43e21711e7a46197acd5645f052be",
        ),
        "just": crate.spec(
            artifact = "bin",
            version = "1.42.4",
        ),
        "miniserve": crate.spec(
            artifact = "bin",
            version = "0.31.0",
        ),
        "mergiraf": crate.spec(
            artifact = "bin",
            version = "0.13.0",
        ),
        # Currently using a fork due to https://github.com/rust-lang/rust-analyzer/issues/20359
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
    stable_as_nightly = True,
    tags = ["manual"],
    vendor_path = "bindeps",
    visibility = ["//visibility:public"],
)
