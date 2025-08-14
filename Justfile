mod? local

# Format all code
fmt:
    bazel run //tools/cargo/fmt:fix
    find . \( -name '*.bazel' -o -name '*.bzl' \) -exec buildifier {} \;

lint:
    bazel run //tools/cargo/clippy:fix

clean *args="--async":
    bazel clean {{args}}
    bazel --output_base=.cache/bazel/coverage clean {{args}}
    bazel --output_base=.cache/bazel/grind clean {{args}}
    bazel --output_base=.cache/bazel/rust_analyzer clean {{args}}

alias coverage := test
test *targets="//...":
    #!/usr/bin/env bash
    set -euo pipefail

    cargo insta reject > /dev/null

    bazel --output_base=.cache/bazel/coverage coverage {{targets}} --//settings:test_insta_force_pass

    snaps=$(find -L $(bazel --output_base=.cache/bazel/coverage info bazel-testlogs) \( -name '*.snap.new' -o -name '*.pending-snap' \))
    # Loop over each found file
    for snap in $snaps; do
        rel_path="${snap#*test.outputs/}"
        # Create the symbolic link inside the target directory
        ln -sf "$(realpath "$snap")" "$(dirname "$rel_path")/$(basename "$rel_path")"
    done

    cargo insta review

vendor:
    cargo update --workspace
    bazel run //tools/cargo/vendor

grind *targets="//...":
    #!/usr/bin/env bash
    set -euo pipefail

    targets=$(bazel query 'kind("nextest_test rule", set({{targets}}))')
    target_runner_value="$(which valgrind) --error-exitcode=1 --leak-check=full --show-leak-kinds=definite --errors-for-leak-kinds=definite --track-origins=yes"
    target_runner_name="CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"

    bazel --output_base=.cache/bazel/grind test ${targets} --test_env="${target_runner_name}=${target_runner_value}" --//settings:test_rustc_flags="--cfg=valgrind"

alias docs := doc

rustdoc_target := "//docs:rustdoc"

doc:
    bazel build {{rustdoc_target}}

alias docs-serve := doc-serve
doc-serve: doc
    miniserve $(bazel info execution_root)/$(bazel cquery --config=wrapper {{rustdoc_target}} --output=files) --index index.html --port 3000

deny:
    bazel run //tools/cargo/deny
