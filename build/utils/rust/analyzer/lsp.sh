#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

output_base="$(bazelisk info output_base)"

bazelisk \
    --output_base="${output_base}_rust_analyzer" \
    run \
    //tools/rust-analyzer
