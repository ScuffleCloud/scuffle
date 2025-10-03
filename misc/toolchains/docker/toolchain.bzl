visibility("public")

def _docker_toolchain_impl(ctx):
    env = {}
    if ctx.attr.docker_path:
        env["DOCKER_CLI"] = ctx.attr.docker_path
    if ctx.attr.docker_host:
        env["DOCKER_HOST"] = ctx.attr.docker_host
    if ctx.attr.containerd_address:
        env["CONTAINERD_ADDRESS"] = ctx.attr.containerd_address
    return [
        platform_common.ToolchainInfo(
            docker_path = ctx.attr.docker_path,
            docker_host = ctx.attr.docker_host,
            containerd_address = ctx.attr.containerd_address,
            env = env,
        ),
    ]

docker_toolchain = rule(
    doc = "A runtime toolchain for shell targets.",
    attrs = {
        "docker_path": attr.string(
            doc = "Absolute path to the docker exectuable",
        ),
        "docker_host": attr.string(
            doc = "The docker host url used to connect to the docker daemon",
        ),
        "containerd_address": attr.string(
            doc = "Address to containerd",
        ),
    },
    implementation = _docker_toolchain_impl,
)
