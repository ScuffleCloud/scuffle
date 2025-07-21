_packages = [
    "//cloud/core",
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
    "//tools/cargo/deny",
    "//tools/cargo/sync-rdme",
    "//build/utils/collect_args",
    "//build/utils/clippy",
    "//build/utils/nextest_runner",
    "//build/utils/process_wrapper",
    "//build/utils/cargo_metadata",
    "//build/utils/rust_doctest_runner",
    "//build/utils/rust_doctest_builder",
    "//build/utils/rust_doctest_common",
    "//build/utils/rustdoc_merger",
    "//build/utils/rustdoc_wrapper",
    "//build/utils/protobuf",
]

cargo_workspace_manifest = "//:Cargo.toml"
cargo_lock = "//:Cargo.lock"

def cargo_manifests(exclude = []):
    return cargo_targets(target = "cargo_toml", exclude = exclude) + [cargo_workspace_manifest]

def _last_part(package):
    return package.split('/')[-1]

def cargo_targets(
    target = "{name}",
    exclude = [],
):
    return [
        package + ":" + target.format(name = _last_part(package))
        for package in _packages if package not in exclude 
    ]
