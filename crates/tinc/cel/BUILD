load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "tinc-cel",
    compile_data = [
        ":Cargo.toml",
    ],
    test = scuffle_test(
        insta = True,
    ),
)
