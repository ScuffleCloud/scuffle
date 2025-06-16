"""Bzlmod module extensions"""

load(
    "//vendor/cargo:crates.bzl",
    "crate_repositories",
)

def _cargo_vendor_impl(module_ctx):
    # This should contain the subset of WORKSPACE.bazel that defines
    # repositories.
    direct_deps = crate_repositories()

    # is_dev_dep is ignored here. It's not relevant for internal_deps, as dev
    # dependencies are only relevant for module extensions that can be used
    # by other MODULES.
    return module_ctx.extension_metadata(
        root_module_direct_deps = [repo.repo for repo in direct_deps],
        root_module_direct_dev_deps = [],
    )

cargo_vendor = module_extension(
    doc = "Vendored crate_universe outputs.",
    implementation = _cargo_vendor_impl,
)
