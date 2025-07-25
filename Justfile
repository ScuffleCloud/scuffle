mod? local

# By default we use the nightly toolchain, however you can override this by setting the RUST_TOOLCHAIN environment variable.
export RUST_TOOLCHAIN := env_var_or_default('RUST_TOOLCHAIN', 'nightly')

# Format all code
fmt:
    bazel run //tools/cargo/fmt:fix
    find . \( -name '*.bazel' -o -name '*.bzl' \) -exec buildifier {} \;

lint *args:
    bazel run //tools/cargo/clippy:fix {{args}}

alias coverage := test
test *args:
    bazel coverage //... {{args}}

coverage-serve:
    miniserve target/llvm-cov/html --index index.html --port 3000

# grind *args:
#     #!/usr/bin/env bash
#     set -euo pipefail

#     # Runs valgrind on the tests.
#     # If there are errors due to tests using global (and not actual memory leaks) then use the
#     # information given by valgrind to replace the "<insert_a_suppression_name_here>" with the actual test name.
#     export RUSTFLAGS="--cfg reqwest_unstable --cfg valgrind"
#     export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --error-exitcode=1 --leak-check=full --gen-suppressions=all --suppressions=$(pwd)/valgrind_suppressions.log"
#     cargo +{{RUST_TOOLCHAIN}} nextest run --all-features --no-fail-fast {{args}}

alias docs := doc
doc *args:
    bazel build //:rustdoc

alias docs-serve := doc-serve
doc-serve: doc
    miniserve target-bazel/bin/rustdoc.rustdoc_merge --index index.html --port 3000

deny *args:
    bazel run //tools/cargo/deny

# readme:
#     #!/usr/bin/env bash
#     set -euo pipefail

#     RUSTDOCFLAGS="-Dwarnings --cfg docsrs --sort-modules-by-appearance --enable-index-page -Zunstable-options"  cargo +nightly sync-rdme --all-features --workspace
