load("//build/utils:package.bzl", "scuffle_package")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    name = "bin",
    crate_name = "nextest_runner_bin",
    crate_type = "bin",
    srcs = ["src/main.rs"],
    deps = [":nextest_runner"],
    tags = ["manual"],
)

scuffle_package(
    crate_name = "nextest_runner_lib",
    srcs = ["src/lib.rs"],
    tags = ["manual"],
)
