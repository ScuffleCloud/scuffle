load("@rules_rust//rust/private:utils.bzl", "transform_deps")
load("@rules_rust//rust/private:rustc.bzl", "collect_deps")
load("@bazel_skylib//lib:paths.bzl", "paths")


def _postcompile_deps_impl(ctx):
    dep_info, _, _ = collect_deps(
        deps = depset(transform_deps(ctx.attr.deps)),
        proc_macro_deps = depset(transform_deps(ctx.attr.proc_macro_deps)),
        aliases = ctx.attr.aliases,
    )

    postcompile_args = {
        "direct": {},
        "search": []
    }

    runfiles = []

    for crate in dep_info.direct_crates.to_list():
        if hasattr(crate, "dep"):
            name = crate.name
            crate_info = crate.dep
        else:
            name = crate.name
            crate_info = crate

        postcompile_args["direct"][name] = crate_info.output.short_path
        runfiles.append(crate_info.output)

    for crate in dep_info.transitive_crates.to_list():
        postcompile_args["search"].append(paths.dirname(crate.output.short_path))
        runfiles.append(crate.output)

    out = ctx.actions.declare_file("{}--postcompile-args.json".format(ctx.label.name))
    ctx.actions.write(
        output = out,
        content = json.encode(postcompile_args),
    )

    return DefaultInfo(
        files = depset([out]),
        runfiles = ctx.runfiles(files = runfiles + [out])
    )

postcompile_deps = rule(
    implementation = _postcompile_deps_impl,
    attrs = {
        "deps": attr.label_list(),
        "proc_macro_deps": attr.label_list(),
        "aliases": attr.label_keyed_string_dict(),
    },
)