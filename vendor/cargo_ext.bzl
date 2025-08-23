"""Bzlmod module extensions"""

load(
    "//vendor/bindeps:crates.bzl",
    bindeps_crate_repositories = "crate_repositories",
)
load(
    "//vendor/cargo:crates.bzl",
    cargo_vendor_crate_repositories = "crate_repositories",
)

def _cargo_vendor_impl(module_ctx):
    # This should contain the subset of WORKSPACE.bazel that defines
    # repositories.
    direct_deps = cargo_vendor_crate_repositories()

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

def _bindeps_impl(module_ctx):
    # This should contain the subset of WORKSPACE.bazel that defines
    # repositories.
    direct_deps = bindeps_crate_repositories()

    # is_dev_dep is ignored here. It's not relevant for internal_deps, as dev
    # dependencies are only relevant for module extensions that can be used
    # by other MODULES.
    return module_ctx.extension_metadata(
        root_module_direct_deps = [repo.repo for repo in direct_deps],
        root_module_direct_dev_deps = [],
    )

bindeps = module_extension(
    doc = "Vendored crate_universe outputs.",
    implementation = _bindeps_impl,
)
