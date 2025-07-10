load("//build/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    aliases = {
        "//cloud/proto": "pb",
    },
    compile_data = [
        "static_policies.cedar"
    ],
    srcs = glob([
        "src/**/*.rs"
    ], exclude = [
        "src/bin/**"
    ]),
    crate_name = "scufflecloud-core",
    deps = [
        "//cloud/proto",
        "//crates/bootstrap",
        "//crates/context",
        "//crates/http",
        "//crates/settings",
        "//crates/signal",
        "//crates/tinc",
    ],
)

scuffle_package(
    name = "bin",
    aliases = {
        "//cloud/proto": "pb",
    },
    srcs = [
        "src/bin/standalone.rs"
    ],
    crate_name = "scufflecloud-core",
    crate_type = "bin",
    deps = [
        ":core",
        "//cloud/proto",
        "//crates/bootstrap",
        "//crates/context",
        "//crates/http",
        "//crates/settings",
        "//crates/signal",
        "//crates/tinc",
    ],
)

