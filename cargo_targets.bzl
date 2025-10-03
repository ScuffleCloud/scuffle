_packages = [
    "//cloud/big-bin",
    "//cloud/ext-traits",
    "//cloud/core",
    "//cloud/core/cedar",
    "//cloud/core/traits",
    "//cloud/core/emails",
    "//cloud/core/db-types",
    "//cloud/email",
    "//cloud/email/traits",
    "//cloud/geo-ip",
    "//cloud/id",
    "//cloud/proto",
    "//crates/aac",
    "//crates/amf0",
    "//crates/av1",
    "//crates/batching",
    "//crates/bootstrap",
    "//crates/bootstrap/derive",
    "//crates/bootstrap-telemetry",
    "//crates/bytes-util",
    "//crates/changelog",
    "//crates/cedar-policy",
    "//crates/cedar-policy/codegen",
    "//crates/context",
    "//crates/expgolomb",
    "//crates/ffmpeg",
    "//crates/flv",
    "//crates/future-ext",
    "//crates/h264",
    "//crates/h265",
    "//crates/http",
    "//crates/metrics",
    "//crates/metrics/derive",
    "//crates/mp4",
    "//crates/nutype-enum",
    "//crates/openapiv3_1",
    "//crates/postcompile",
    "//crates/pprof",
    "//crates/rtmp",
    "//crates/settings",
    "//crates/signal",
    "//crates/tinc",
    "//crates/tinc/build",
    "//crates/tinc/cel",
    "//crates/tinc/derive",
    "//crates/tinc/integration",
    "//crates/tinc/pb-prost",
    "//crates/transmuxer",
    "//dev-tools/xtask",
    "//tools/cargo/clippy",
    "//tools/cargo/sync-readme",
    "//misc/utils/protobuf/file_concat",
    "//misc/utils/rust/clippy",
    "//misc/utils/rust/diesel_migration/runner",
    "//misc/utils/rust/diesel_migration/test",
    "//misc/utils/rust/test_runner",
    "//misc/utils/rust/test_runner/lib",
    "//misc/utils/rust/doc/test/runner",
    "//misc/utils/rust/doc/test/builder",
    "//misc/utils/rust/doc/test/common",
    "//misc/utils/rust/doc/merger",
    "//misc/utils/rust/doc/wrapper",
    "//misc/utils/rust/analyzer/check",
    "//misc/utils/rust/analyzer/discover",
    "//misc/utils/rust/sync_readme",
    "//misc/utils/rust/sync_readme/common",
    "//misc/utils/rust/sync_readme/test_runner",
]

cargo_workspace_manifest = "//:Cargo.toml"
cargo_lock = "//:Cargo.lock"

def _last_part(package):
    return package.split("/")[-1]

def cargo_targets(
        target = "{name}",
        exclude = []):
    return [
        package + ":" + target.format(name = _last_part(package))
        for package in _packages
        if package not in exclude
    ]
