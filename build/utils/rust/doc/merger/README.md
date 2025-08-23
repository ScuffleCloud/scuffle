<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# rustdoc_merger
<!-- sync-readme ]] -->

<!-- sync-readme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
This is a binary helper to merge rustdoc html outputs

Normally rustdoc is run sequentially outputting to a single output directory. This goes against how
bazel works and causes issues since every time we change one crate we would need to regenerate the docs for all crates.

This tool fixes this problem by essentially performing the same steps that rustdoc would do when merging except does it
in a bazel-like way.
<!-- sync-readme ]] -->
