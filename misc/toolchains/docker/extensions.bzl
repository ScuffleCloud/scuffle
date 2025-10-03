load("@bazel_features//:features.bzl", "bazel_features")
load("//misc/toolchains/docker:docker_config.bzl", "docker_config")

def _docker_configure_impl(module_ctx):
    docker_config(name = "local_docker_config")
    if bazel_features.external_deps.extension_metadata_has_reproducible:
        return module_ctx.extension_metadata(reproducible = True)
    return None

docker_configure = module_extension(implementation = _docker_configure_impl)
