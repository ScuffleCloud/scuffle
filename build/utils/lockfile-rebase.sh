#!/usr/bin/env bash
set -euo pipefail

ancestor="$1"
current="$2"
other="$3"
marker_size="$4"
file="$5"

case "$file" in
  *Cargo.lock)
    echo "Resolving Cargo.lock conflict by taking base and regenerating..."
    cp "$other" "$file"
    cargo update --workspace >/dev/null 2>&1 || true
    ;;
  *pnpm-lock.yaml)
    echo "Resolving pnpm-lock.yaml conflict by taking base and regenerating..."
    cp "$other" "$file"
    pnpm install --lockfile-only >/dev/null 2>&1 || true
    ;;
  *MODULE.bazel.lock)
    echo "Resolving MODULE.bazel.lock conflict by taking base and regenerating..."
    cp "$other" "$file"
    bazel shutdown 2>&1 || true
    bazel fetch //... --lockfile_mode=off 2>&1 || true
    bazel fetch //... --lockfile_mode=update 2>&1 || true
    ;;
  *)
    echo "Unknown lockfile type: $file" >&2
    exit 1
    ;;
esac

exit 0
