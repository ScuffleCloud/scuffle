load("//build/utils:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-bootstrap-derive",
    crate_type = "proc_macro",
    test = scuffle_test(
        insta = True,
    ),
)
