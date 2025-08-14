#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

bazelisk \
    --output_base=.cache/bazel/rust_analyzer \
    run \
    --config=wrapper \
    //tools/rust-analyzer
