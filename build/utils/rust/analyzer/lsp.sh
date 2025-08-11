#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

bazelisk \
    run \
    --config=wrapper \
    //tools/rust-analyzer
