load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-bootstrap-derive",
    crate_type = "proc_macro",
    test = scuffle_test(
        insta = True,
    ),
)
