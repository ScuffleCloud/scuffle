# done

- cargo test + coverage
    ```
    bazel coverage //...
    ```

- cargo fmt + fix
    ```
    bazel test //... # for checking fmt
    bazel run //tools/cargo/fmt:fix # for fixing fmt
    ```

- cargo clippy + fix
    ```
    bazel test //... # for checking clippy
    bazel run //tools/cargo/clippy:fix # for fixing clippy
    ```

- cargo deny

    This is not a test because it depends on external network access to load the advisories.

    ```
    bazel run //tools/cargo/deny # for checking deny
    ```

- cargo doc

    Docs are build with

    ```
    bazel build //...
    ```

# easy to do

- cargo syncreadme
- rust analyzer
- cargo valgrind

# hard to do

- xtask release-checks
- powerset
