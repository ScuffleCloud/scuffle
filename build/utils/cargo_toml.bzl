def _cargo_toml_impl(ctx):
    srcs = []
    for src in ctx.attr.srcs:
        srcs.extend(src.files.to_list())

    return DefaultInfo(
        files = ctx.attr.manifest.files,
        runfiles = ctx.runfiles(files = srcs),
    )

_cargo_toml = rule(
    implementation = _cargo_toml_impl,
    attrs = {
        "manifest": attr.label(
            doc = "Cargo.toml file",
            allow_single_file = True,
            mandatory = True,
        ),
        "srcs": attr.label_list(
            doc = "Additional runfile srcs",
            allow_empty = True,
            allow_files = True,
        ),
    },
)

def cargo_toml(
    name = None,
    manifest = None,
    srcs = None,
    visibility = None,
):
    if name == None:
        name = "cargo_toml"
    if manifest == None:
        manifest = ":Cargo.toml"
    if srcs == None:
        srcs = native.glob([
            "src/main.rs",
            "src/lib.rs",
            "src/bin/**/*.rs",
            "benches/**/*.rs",
            "examples/**/*.rs",
            "tests/**/*.rs",
        ], allow_empty = True)
    if visibility == None:
        visibility = ["//visibility:public"]

    _cargo_toml(
        name = name,
        manifest = manifest,
        srcs = srcs,
        visibility = visibility,
    )
