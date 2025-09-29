mod? local

default:
    just --list

# this should be kept in sync with
# .github/workflows/ci-check-fmt.yaml

fmt:
    bazel run //tools/cargo/fmt:fix
    buildifier $(git ls-files "*.bzl" "*.bazel" | xargs ls 2>/dev/null)
    dprint fmt
    buf format -w --disable-symlinks --debug
    just --unstable --fmt
    shfmt -w .
    git ls-files "*.cedar" | xargs ls 2>/dev/null | xargs -I {} cedar format -w -p {}

lint:
    bazel run //tools/cargo/clippy:fix
    pnpm lint:fix --ui=stream

clean *args="--async":
    #!/usr/bin/env bash
    set -exuo pipefail

    output_base=$(bazel info output_base)

    rm -r "${output_base}/.scripts"
    bazel --output_base="${output_base}" clean {{ args }}
    bazel --output_base="${output_base}_coverage" clean {{ args }}
    bazel --output_base="${output_base}_grind" clean {{ args }}
    bazel --output_base="${output_base}_rust_analyzer" clean {{ args }}

clear-tool-cache:
    #!/usr/bin/env bash
    output_base=$(bazel info output_base)
    rm -r "${output_base}/.scripts"

run bin *args:
    #!/usr/bin/env bash

    if [ {{ bin }} == "core" ]; then
        bazel run //cloud/core:bin -- {{ args }}
    elif [ {{ bin }} == "email" ]; then
        bazel run //cloud/email:bin -- {{ args }}
    else
        echo "Unknown binary: {{ bin }}"
        exit 1
    fi

generate-mtls-certs:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p local/mtls

    # Generate root CA
    openssl genpkey -out local/mtls/root_key.pem -algorithm ED25519
    openssl req -x509 -new -key local/mtls/root_key.pem \
        -subj "/CN=scufflecloud-mtls-root" \
        -days 365 -out local/mtls/root_cert.pem

    # Generate core cert signed by root CA
    openssl genpkey -out local/mtls/scufflecloud_core_key.pem -algorithm ED25519
    openssl req -new -key local/mtls/scufflecloud_core_key.pem \
        -subj "/CN=scufflecloud-core-mtls" \
        -addext "subjectAltName=DNS:localhost" \
        -out local/mtls/scufflecloud_core_csr.pem

    # Sign core cert with root CA
    openssl x509 -req \
        -in local/mtls/scufflecloud_core_csr.pem \
        -CA local/mtls/root_cert.pem \
        -CAkey local/mtls/root_key.pem \
        -CAcreateserial -days 365 \
        -out local/mtls/scufflecloud_core_cert.pem \
        -copy_extensions copy

    # Generate email cert signed by root CA
    openssl genpkey -out local/mtls/scufflecloud_email_key.pem -algorithm ED25519
    openssl req -new -key local/mtls/scufflecloud_email_key.pem \
        -subj "/CN=scufflecloud-email-mtls" \
        -addext "subjectAltName=DNS:localhost" \
        -out local/mtls/scufflecloud_email_csr.pem

    # Sign email cert with root CA
    openssl x509 -req \
        -in local/mtls/scufflecloud_email_csr.pem \
        -CA local/mtls/root_cert.pem \
        -CAkey local/mtls/root_key.pem \
        -CAcreateserial -days 365 \
        -out local/mtls/scufflecloud_email_cert.pem \
        -copy_extensions copy

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
