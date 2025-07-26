#!/usr/bin/env bash

set -euo pipefail

bazelisk \
    run \
    --config=wrapper \
    @rules_rust//tools/rust_analyzer:discover_bazel_rust_project -- \
    --bazel_startup_option=--output_base=~/ide_bazel \
    --bazel_arg=--watchfs
