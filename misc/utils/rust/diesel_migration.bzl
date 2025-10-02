load("@rules_img//img:load.bzl", "image_load")

DieselMigrationInfo = provider(
    doc = "The result of the diesel migration.",
    fields = {
        "result": "The result of the diesel migration.",
        "schemas": "The schemas of the diesel migration.",
    },
)

def _diesel_migration_impl(ctx):
    result = ctx.actions.declare_file(ctx.label.name + ".result.json")

    ctx.actions.run(
        executable = ctx.executable._diesel_migration_tool,
        arguments = [schema.path for schema in ctx.files.schemas],
        inputs = ctx.files.data + [ctx.file.config_file],
        outputs = [result],
        env = {
            "DIESEL_CLI_TOOL": ctx.executable._diesel_cli_tool.path,
            "DATABASE_IMAGE_LOAD_TOOL": ctx.executable.database_image_load.path,
            "DIESEL_CONFIG_FILE": ctx.file.config_file.path,
            "OUTPUT_FILE": result.path,
        },
        use_default_shell_env = True,
        tools = [
            ctx.executable._diesel_cli_tool,
            ctx.executable.database_image_load,
        ],
    )

    return [
        DefaultInfo(files = depset([result])),
        OutputGroupInfo(
            diesel_schema = [result],
        ),
        DieselMigrationInfo(
            result = result,
            schemas = ctx.files.schemas,
        ),
    ]

_diesel_migration = rule(
    implementation = _diesel_migration_impl,
    attrs = {
        "data": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "schemas": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "database_image_load": attr.label(
            mandatory = True,
            executable = True,
            cfg = "exec",
        ),
        "config_file": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "_diesel_cli_tool": attr.label(
            default = "//tools/diesel",
            executable = True,
            cfg = "exec",
        ),
        "_diesel_migration_tool": attr.label(
            default = "//misc/utils/rust/diesel_migration/runner",
            executable = True,
            cfg = "exec",
        ),
    },
)

def diesel_migration(
        name,
        data,
        schemas,
        database_image,
        config_file):
    image_load(
        name = "{}_load".format(name),
        image = database_image,
        tag = "bazel-{}:{}".format(native.package_name().replace("/", "-"), name),
    )

    _diesel_migration(
        name = name,
        data = data,
        schemas = schemas,
        database_image_load = "{}_load".format(name),
        config_file = config_file,
    )

def _get_rlocation_path(ctx, file):
    # Determine the workspace name for this file
    if file.owner and file.owner.workspace_name:
        # External workspace file
        workspace_name = file.owner.workspace_name
    else:
        # Main workspace file - use the current workspace name
        workspace_name = ctx.workspace_name

    # Construct the full rlocation path
    return workspace_name + "/" + file.short_path

def _diesel_migration_test_impl(ctx):
    test_binary = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.symlink(output = test_binary, target_file = ctx.executable._diesel_migration_test)

    migration = ctx.attr.diesel_migration[DieselMigrationInfo]

    return [
        DefaultInfo(executable = test_binary, files = depset([test_binary]), runfiles = ctx.runfiles(files = [migration.result] + migration.schemas)),
        RunEnvironmentInfo(
            environment = {
                "RESULT_PATH": _get_rlocation_path(ctx, migration.result),
                "SCHEMAS_PATHS": ";".join([schema.path for schema in migration.schemas]),
                "SCHEMAS_SHORT_PATHS": ";".join([_get_rlocation_path(ctx, schema) for schema in migration.schemas]),
            },
        ),
    ]

diesel_migration_test = rule(
    implementation = _diesel_migration_test_impl,
    attrs = {
        "diesel_migration": attr.label(
            providers = [DieselMigrationInfo],
        ),
        "_diesel_migration_test": attr.label(
            default = "//misc/utils/rust/diesel_migration/test",
            executable = True,
            cfg = "exec",
        ),
    },
    test = True,
)
