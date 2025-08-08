#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

bazelisk \
    run \
    --config=wrapper \
    //build/utils/rust/analyzer/check \
    -- \
    --config=wrapper 2> /dev/null
