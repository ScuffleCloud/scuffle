mod? local

# this should be kept in sync with
# .github/workflows/cargo-update-pr.yaml

bzlmod:
    # https://github.com/bazelbuild/bazel/issues/20477
    bazel shutdown
    bazel fetch //... --lockfile_mode=off
    bazel fetch //... --lockfile_mode=update

# this should be kept in sync with
# .github/workflows/ci-check-fmt.yaml

fmt:
    bazel run //tools/cargo/fmt:fix
    buildifier $(git ls-files "*.bzl" "*.bazel" | xargs ls 2>/dev/null)
    dprint fmt
    buf format -w --disable-symlinks --debug
    just --unstable --fmt

lint:
    bazel run //tools/cargo/clippy:fix
    pnpm lint:fix --ui=stream

clean *args="--async":
    #!/usr/bin/env bash
    set -exuo pipefail

    output_base=$(bazel info output_base)

    bazel --output_base="${output_base}" clean {{ args }}
    bazel --output_base="${output_base}_coverage" clean {{ args }}
    bazel --output_base="${output_base}_grind" clean {{ args }}
    bazel --output_base="${output_base}_rust_analyzer" clean {{ args }}

run core *args:
    bazel run //cloud/core:bin -- {{ args }}

alias coverage := test

sync-rdme:
    bazel run //tools/cargo/sync-readme:fix

test *targets="//...":
    #!/usr/bin/env bash
    set -exuo pipefail

    cargo insta reject > /dev/null

    output_base=$(bazel info output_base)

    targets=$(bazel query 'tests(set({{ targets }}))')

    bazel --output_base="${output_base}_coverage" coverage ${targets} --//settings:test_insta_force_pass --skip_incompatible_explicit_targets

    test_logs=$(bazel --output_base="${output_base}_coverage" info bazel-testlogs)

    snaps=$(find -L "${test_logs}" \( -name '*.snap.new' -o -name '*.pending-snap' \))
    # Loop over each found file
    for snap in $snaps; do
        rel_path="${snap#*test.outputs/}"
        # Create the symbolic link inside the target directory
        ln -sf "$(realpath "$snap")" "$(dirname "$rel_path")/$(basename "$rel_path")"
    done

    cargo insta review

    rm lcov.info || true
    ln -s "$(bazel --output_base="${output_base}_coverage" info output_path)"/_coverage/_coverage_report.dat lcov.info

# this should be kept in sync with
# .github/workflows/ci-check-vendor.yaml

alias vendor := lockfile

lockfile:
    cargo update --workspace
    bazel run //vendor:cargo_vendor
    bazel run //vendor:bindeps
    pnpm install --lockfile-only

grind *targets="//...":
    #!/usr/bin/env bash
    set -euxo pipefail

    output_base=$(bazel info output_base)
    targets=$(bazel query 'kind("nextest_test rule", set({{ targets }}))')

    bazel --output_base="${output_base}_grind" test ${targets} --//settings:test_rustc_flags="--cfg=valgrind" --//settings:test_valgrind --skip_incompatible_explicit_targets

alias docs := doc

rustdoc_target := "//docs:rustdoc"

doc:
    bazel build {{ rustdoc_target }}

alias docs-serve := doc-serve

doc-serve: doc
    miniserve "$(bazel info execution_root)"/"$(bazel cquery --config=wrapper {{ rustdoc_target }} --output=files)" --index index.html --port 3000

deny:
    bazel run //tools/cargo/deny check
