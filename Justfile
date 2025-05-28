mod? local

# By default we use the nightly toolchain, however you can override this by setting the RUST_TOOLCHAIN environment variable.
export RUST_TOOLCHAIN := env_var_or_default('RUST_TOOLCHAIN', 'nightly')

# An alias for cargo xtask check
powerset *args:
    cargo +{{RUST_TOOLCHAIN}} xtask powerset {{args}}

# An alias for cargo fmt --all
fmt *args:
    cargo +{{RUST_TOOLCHAIN}} fmt --all {{args}}

lint *args:
    cargo +{{RUST_TOOLCHAIN}} clippy --fix --allow-dirty --allow-staged --all-features --all-targets {{args}} -- -Aclippy::collapsible_if

alias coverage := test
test *args:
    #!/usr/bin/env bash
    set -euo pipefail

    # Download ISOBMFF conformance samples
    if [ ! -d ./assets/isobmff_conformance ]; then
        echo "Downloading ISOBMFF conformance samples..."
        mkdir -p ./assets/isobmff_conformance
        curl -Lo ./assets/isobmff_conformance/files.json https://github.com/MPEGGroup/FileFormatConformance/releases/download/r20250526/files.json
        curl -Lo ./assets/isobmff_conformance/conformance-files.tar.gz https://github.com/MPEGGroup/FileFormatConformance/releases/download/r20250526/conformance-files.tar.gz
        tar xf ./assets/isobmff_conformance/conformance-files.tar.gz --directory=./assets/isobmff_conformance
    else
        echo "ISOBMFF conformance samples already downloaded."
    fi

    INSTA_FORCE_PASS=1 cargo +{{RUST_TOOLCHAIN}} llvm-cov clean --workspace
    INSTA_FORCE_PASS=1 cargo +{{RUST_TOOLCHAIN}} llvm-cov nextest --include-build-script --no-report --all-features -- {{args}}
    # Coverage for doctests is currently broken in llvm-cov.
    # Once it fully works we can add the `--doctests` flag to the test and report command again.
    cargo +{{RUST_TOOLCHAIN}} llvm-cov test --doc --no-report --all-features {{args}}

    # Do not generate the coverage report on CI
    cargo insta review
    cargo +{{RUST_TOOLCHAIN}} llvm-cov report --include-build-script --lcov --output-path ./lcov.info
    cargo +{{RUST_TOOLCHAIN}} llvm-cov report --include-build-script --html

doctest *args:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo +{{RUST_TOOLCHAIN}} llvm-cov test --doc --no-report --all-features {{args}}

coverage-serve:
    miniserve target/llvm-cov/html --index index.html --port 3000

grind *args:
    #!/usr/bin/env bash
    set -euo pipefail

    # Runs valgrind on the tests.
    # If there are errors due to tests using global (and not actual memory leaks) then use the
    # information given by valgrind to replace the "<insert_a_suppression_name_here>" with the actual test name.
    export RUSTFLAGS="--cfg reqwest_unstable --cfg valgrind"
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --error-exitcode=1 --leak-check=full --gen-suppressions=all --suppressions=$(pwd)/valgrind_suppressions.log"
    cargo +{{RUST_TOOLCHAIN}} nextest run --all-features --no-fail-fast {{args}}

alias docs := doc
doc *args:
    #!/usr/bin/env bash
    set -euo pipefail

    # `--cfg docsrs` enables us to write feature hints in the form of `#[cfg_attr(docsrs, doc(cfg(feature = "some-feature")))]`
    # `--enable-index-page` makes the command generate an index page which lists all crates (unstable)
    # `--generate-link-to-definition` generates source code links (unstable)
    # `--sort-modules-by-appearance` sorts modules by the order they were defined in (unstable)
    # `-D warnings` disallow all warnings
    # `-Zunstable-options` enables unstable options (for the `--enable-index-page` flag)
    export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -Dwarnings --cfg docsrs --sort-modules-by-appearance --generate-link-to-definition --enable-index-page -Zunstable-options"
    cargo +{{RUST_TOOLCHAIN}} doc --no-deps --all-features {{args}}

alias docs-serve := doc-serve
doc-serve: doc
    miniserve target/doc --index index.html --port 3000

deny *args:
    cargo +{{RUST_TOOLCHAIN}} deny {{args}} --all-features check

workspace-hack:
    cargo +{{RUST_TOOLCHAIN}} hakari manage-deps
    cargo +{{RUST_TOOLCHAIN}} hakari generate

alias version-check := check-versions
alias check-version := check-versions
alias versions-check := check-versions
check-versions:
    release-plz update --disable-dependant-updates --no-changelog --check-only --exit-status

readme:
    #!/usr/bin/env bash
    set -euo pipefail

    RUSTDOCFLAGS="-Dwarnings --cfg docsrs --sort-modules-by-appearance --enable-index-page -Zunstable-options"  cargo +nightly sync-rdme --all-features --workspace
