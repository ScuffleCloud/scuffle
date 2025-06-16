#!/bin/env bash
set -euo pipefail

BINARY=$(realpath %%BINARY%%)
LIB_DIR="$(dirname $(dirname $BINARY))/lib"

# Set up library path and execute
export LD_LIBRARY_PATH="$LIB_DIR:${LD_LIBRARY_PATH:-}"
exec "$BINARY" "$@"
