#!/usr/bin/env bash
set -euo pipefail

protoc \
  --plugin ./node_modules/.bin/protoc-gen-ts \
  --ts_out ./src/generated/proto \
  --ts_opt long_type_string \
  --ts_opt optimize_code_size \
  --proto_path ../crates \
  --proto_path ../cloud/proto/pb \
  ../cloud/proto/pb/scufflecloud/core/v1/*

cd .. && dprint fmt
