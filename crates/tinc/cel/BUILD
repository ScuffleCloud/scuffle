load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "tinc-cel",
    compile_data = [
        ":Cargo.toml",
    ],
    test = scuffle_test(
        insta = True,
    ),
)
