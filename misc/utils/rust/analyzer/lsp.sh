#!/usr/bin/env bash

set -euo pipefail

SCRIPT="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(dirname "${SCRIPT}")"

cd "${SCRIPT_DIR}/../../../../"

export PATH="$(pwd)/tools/scripts:$PATH"

exec rust-analyzer $@
