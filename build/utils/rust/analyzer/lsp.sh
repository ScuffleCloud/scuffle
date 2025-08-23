#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

SCRIPT="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(dirname "${SCRIPT}")"

cd "${SCRIPT_DIR}/../../../../"

output_base="$(bazelisk info output_base)"

echo "Running rust-analyzer" >&2

bazelisk \
    --output_base="${output_base}_rust_analyzer" \
    --preemptible \
    run \
    --config=wrapper \
    //tools/rust-analyzer -- $@
