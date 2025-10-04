_TOOLCHAIN_TEMPLATE = """
load("@//misc/toolchains/docker:toolchain.bzl", "docker_toolchain")

docker_toolchain(
    name = "host_docker",
    docker_path = {docker_path},
    docker_host = {docker_host},
    containerd_address = {containerd_address},
)

toolchain(
    name = "host_docker_toolchain",
    toolchain = ":host_docker",
    toolchain_type = "@//misc/toolchains/docker:toolchain_type",
)
"""

def _docker_config_impl(repository_ctx):
    repository_ctx.file("BUILD", _TOOLCHAIN_TEMPLATE.format(
        docker_path = repr(_find_docker(repository_ctx)),
        docker_host = repr(repository_ctx.os.environ.get("DOCKER_HOST")),
        containerd_address = repr(repository_ctx.os.environ.get("CONTAINERD_ADDRESS")),
    ))

def _find_docker(ctx):
    return (
        ctx.os.environ.get("BAZEL_DOCKER") or
        ctx.which("docker") or
        ctx.which("podman") or
        ctx.which("docker.exe") or
        ctx.which("podman.exe")
    )

docker_config = repository_rule(
    environ = [
        "DOCKER_HOST",
        "CONTAINERD_ADDRESS",
        "PATH",
        "BAZEL_DOCKER",
    ],
    configure = True,
    implementation = _docker_config_impl,
)
