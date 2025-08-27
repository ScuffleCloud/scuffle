#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

output_base="$(bazelisk info output_base)"

bazelisk \
    --output_base="${output_base}_rust_analyzer" \
    run \
    --config=wrapper \
    //misc/utils/rust/analyzer/check \
    -- \
    --config=wrapper --output-base="${output_base}_rust_analyzer" 2> /dev/null
