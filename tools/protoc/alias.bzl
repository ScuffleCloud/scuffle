load(
    "@rules_proto//proto:proto_common.bzl",
    proto_toolchains = "toolchains",
)

_PROTO_TOOLCHAIN_TYPE = "@rules_proto//proto:toolchain_type"

def _protoc_alias_impl(ctx):
    proto_toolchain = proto_toolchains.find_toolchain(
        ctx,
        legacy_attr = "_legacy_proto_toolchain",
        toolchain_type = _PROTO_TOOLCHAIN_TYPE,
    )

    protoc = proto_toolchain.proto_compiler

    symlink = ctx.actions.declare_file(protoc.executable.basename)
    ctx.actions.symlink(output = symlink, target_file = protoc.executable)

    return [
        DefaultInfo(executable = symlink),
    ]

protoc_alias = rule(
    implementation = _protoc_alias_impl,
    toolchains = [_PROTO_TOOLCHAIN_TYPE],
    executable = True,
)
