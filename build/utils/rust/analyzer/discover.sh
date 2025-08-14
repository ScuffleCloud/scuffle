#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

output_base="$(bazelisk info output_base 2> /dev/null)"

bazelisk \
    --output_base="${output_base}_rust_analyzer" \
    run \
    --config=wrapper \
    //build/utils/rust/analyzer/discover \
    -- \
    --bazel_arg=--config=wrapper --bazel_startup_option=--output_base="${output_base}_rust_analyzer" 2> /dev/null
