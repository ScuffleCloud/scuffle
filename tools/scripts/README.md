# Bazel Tools Wrappers

This directory contains a set of scripts which wrap various tools we use in our bazel builds, such as `cargo`, `rustc`, `rust-analyzer`, `clippy`, `cargo-deny`, etc.

We do this by invoking bazel to run the tool which pulls it in via our toolchain rules, ensuring that everyone uses the same version of the tool, and that it is built in a hermetic environment.
