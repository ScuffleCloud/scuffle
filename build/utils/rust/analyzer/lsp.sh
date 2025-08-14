#!/usr/bin/env bash

set -euo pipefail

export BAZEL="$(which bazelisk)"

SCRIPT="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(dirname "${SCRIPT}")"

cd "${SCRIPT_DIR}/../../../../"

output_base="$(bazelisk info output_base)"

export RA_LOG="lsp_server=debug"

bazelisk \
    --output_base="${output_base}_rust_analyzer" \
    run \
    --config=wrapper \
    //tools/rust-analyzer -- $@
