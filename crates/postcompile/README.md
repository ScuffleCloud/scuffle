<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# postcompile
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/postcompile/0.3.3.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/postcompile/0.3.3)
[![crates.io](https://img.shields.io/badge/crates.io-v0.3.3-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/postcompile/0.3.3)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/postcompile/0.3.3.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/postcompile/0.3.3.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A crate which allows you to compile Rust code at runtime (hence the name
`postcompile`).

What that means is that you can provide the input to `rustc` and then get
back the expanded output, compiler errors, warnings, etc.

This is particularly useful when making snapshot tests of proc-macros, look
below for an example with the `insta` crate.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Usage

````rust,standalone_crate,test_harness
# macro_rules! assert_snapshot {
#     ($expr:expr) => { $expr };
# }
#[test]
fn some_cool_test() {
    assert_snapshot!(postcompile::compile!({
        #![allow(unused)]

        #[derive(Debug, Clone)]
        struct Test {
            a: u32,
            b: i32,
        }

        const TEST: Test = Test { a: 1, b: 3 };
    }));
}

#[test]
fn some_cool_test_extern() {
    assert_snapshot!(postcompile::compile_str!(include_str!("some_file.rs")));
}

#[test]
fn test_inside_test() {
    assert_snapshot!(postcompile::compile!(
        postcompile::config! {
            test: true,
        },
        {
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            #[test]
            fn test_add() {
                assert_eq!(add(1, 2), 3);
            }
        },
    ));
}

#[test]
fn test_inside_test_with_tokio() {
    assert_snapshot!(postcompile::compile!(
        postcompile::config! {
            test: true,
            dependencies: vec![
                postcompile::Dependency::version("tokio", "1").feature("full")
            ]
        },
        {
            async fn async_add(a: i32, b: i32) -> i32 {
                a + b
            }

            #[tokio::test]
            async fn test_add() {
                assert_eq!(async_add(1, 2).await, 3);
            }
        },
    ));
}
````

### Features

* Cached builds: This crate reuses the cargo build cache of the original
  crate so that only the contents of the macro are compiled & not any
  additional dependencies.
* Coverage: This crate works with [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov)
  out of the box, which allows you to instrument the proc-macro expansion.
* Testing: You can define tests with the `#[test]` macro and the tests will run on the generated code.

### Alternatives

* [`compiletest_rs`](https://crates.io/crates/compiletest_rs): This crate is
  used by the Rust compiler team to test the compiler itself. Not really
  useful for proc-macros.
* [`trybuild`](https://crates.io/crates/trybuild): This crate is an
  all-in-one solution for testing proc-macros, with built in snapshot
  testing.
* [`ui_test`](https://crates.io/crates/ui_test): Similar to `trybuild` with
  a slightly different API & used by the Rust compiler team to test the
  compiler itself.

#### Differences

The other libraries are focused on testing & have built in test harnesses.
This crate takes a step back and allows you to compile without a testing
harness. This has the advantage of being more flexible, and allows you to
use whatever testing framework you want.

In the examples above I showcase how to use this crate with the `insta`
crate for snapshot testing.

### Limitations

Please note that this crate does not work inside a running compiler process
(inside a proc-macro) without hacky workarounds and complete build-cache
invalidation.

This is because `cargo` holds a lock on the build directory and that if we
were to compile inside a proc-macro we would recursively invoke the
compiler.

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
