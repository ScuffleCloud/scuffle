# Scuffle Contribution Guide

## Code of Conduct

We have a [Code of Conduct](./CODE_OF_CONDUCT.md) that we expect all contributors to follow. Please read it before contributing.

## Developer Environment

All developers need to have a working rust developer environment setup. You can install rust via [rustup](https://rustup.rs/).

After that you can run the following command to install all the other tools.

```bash
cargo xtask dev-tools
```

<details>
<summary>If you want to install the tools manually you can do so with the following commands.</summary>

You should install both the stable and nightly rust toolchains.

```bash
rustup install stable
rustup install nightly
```

After installing rust you should also install a few components.

```bash
rustup component add clippy
rustup component add rustfmt
rustup component add llvm-tools-preview
rustup component add rust-src
rustup component add rust-docs
```

Then we need to install `cargo-binstall` to be able to install the other crates.

```bash
cargo install cargo-binstall
```

then all of the other crates can be installed with `cargo binstall`.

```bash
cargo binstall just cargo-llvm-cov cargo-nextest cargo-insta cargo-hakari miniserve taplo
```

</details>

### FFmpeg

You need to have ffmpeg 7.1, with dev headers and shared libraries, installed.

#### Package Managers

Package managers often have an out-dated version of ffmpeg, so make sure you check your package manager before installing ffmpeg.

For MacOS (or Linux) you can install ffmpeg via homebrew.

```bash
brew install ffmpeg
```

For Windows you can install ffmpeg via chocolatey or scoop.

```bash
choco install ffmpeg
scoop install ffmpeg
```

#### Pre-built Binaries (Windows / Linux)

However its far easier to download pre-built binaries from [here](https://github.com/BtbN/FFmpeg-Builds/releases/tag/latest) (linux or windows).

| Platform | Download |
|----------|----------|
| Windows | [**`ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip`**](https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip) |
| Windows ARM | [**`ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip`**](https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-winarm64-gpl-shared-7.1.zip) |
| Linux | [**`ffmpeg-n7.1-latest-linux64-gpl-shared-7.1.zip`**](https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-linux64-gpl-shared-7.1.zip) |
| Linux ARM | [**`ffmpeg-n7.1-latest-linuxarm64-gpl-shared-7.1.tar.xz`**](https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-linuxarm64-gpl-shared-7.1.tar.xz) |

#### Installing from source

You can download it from source and build it with at least the following libraries:

- libx264
- libx265
- libvpx
- libopus
- libdav1d

For more information on building ffmpeg from source, you can refer to the [FFmpeg Documentation](https://trac.ffmpeg.org/wiki/CompilationGuide).

### Valgrind (Linux / WSL Only)

You need to have valgrind (at least version 3.24) installed.

You can either build it from source or download it from our pre-built binaries [here](https://github.com/ScuffleCloud/valgrind-builds/releases/tag/latest).

### Common Build Issues

#### Cargo build fails with unable to find ffmpeg installation.

Make sure ffmpeg is actually installed. If you are on linux or macos that you have `pkg-config` installed.

```bash
# Ubuntu / Debian
sudo apt-get install pkg-config

# MacOS
brew install pkg-config
```

If this fails, try setting the following environment variables:

These assume you have the path to ffmpeg installed in `${FFMPEG_ROOT}` and that you have an environment variable `FFMPEG_ROOT` set.

##### Linux / MacOS

```bash
export FFMPEG_PKG_CONFIG_PATH="${FFMPEG_ROOT}/lib/pkgconfig"
export FFMPEG_LIBS_DIR="${FFMPEG_ROOT}/lib"
export FFMPEG_INCLUDE_DIR="${FFMPEG_ROOT}/include"
export PATH="${FFMPEG_ROOT}/bin:${FFMPEG_ROOT}/lib:${FFMPEG_ROOT}/include:${PATH}"
```

##### Windows

```ps1
$env:FFMPEG_PKG_CONFIG_PATH="${$env:FFMPEG_ROOT}\lib\pkgconfig"
$env:FFMPEG_LIBS_DIR="${$env:FFMPEG_ROOT}\lib"
$env:FFMPEG_INCLUDE_DIR="${$env:FFMPEG_ROOT}\include"
$env:PATH="${$env:FFMPEG_ROOT}\bin;${$env:FFMPEG_ROOT}\lib;${$env:FFMPEG_ROOT}\include;${env:PATH}"
```

On windows you need to include not just the binary directory but also the lib and include directories in your path, so that the linker can find the libraries.

### Protobuf-compiler

In `scuffle-tinc` we use the protobuf compiler which you can install via the following instructions: https://protobuf.dev/installation/


## Local Commnads

| Command | Description |
|---------|-------------|
| `just test` | Run all tests |
| `just grind` | Run tests with valgrind |
| `just lint` | Lint the code & try auto-fix linting errors |
| `just format` | Format the code |
| `just workspace-hack` | Update the workspace hack cache, when adding / removing dependencies |
| `just powerset <command>` | Run the powerset tests for a command |
| `just deny` | Check that all dependencies have allowed licenses |
| `just docs` | Build the docs |
| `just docs-serve` | Serve the docs locally |
| `just coverage-serve` | Serve the coverage report locally |

## CLA

We require all contributors to sign a [Contributor License Agreement](./CLA.md) before we can accept any contributions.

To sign the CLA, please head over to [cla.scuffle.cloud](https://cla.scuffle.cloud) and sign the CLA.

## Making a Pull Request

### Commit Messages

We do not squash any commits, we prefer if commits are meaningful and descriptive but this is not required.

### Pull Request Body

The body of the pull request should be a summary of the changes made in the pull request as well as a list of the tickets & issues that are affected by the changes. 

### Pull Request Title

The title of the pull request should be a short and descriptive title of the changes made in the pull request.

### Changelogs

We use a custom changelog format, you can read more about it [here](./changes.d/README.md).

### Documentation

We require that all public methods, types, and functions are documented, with ideally doc examples on how to use the method when applicable. 

### CI Jobs

#### Formatting

We have a ci job that will check that the code is formatted correctly, you can run `just format` to format the code locally.

#### Linting

We have a ci job that will check that the code is linted correctly, you can run `just lint` to lint the code locally.

##### Powersets

A common issue with rust crates with many features is that some combinations of the features do not work together but are expected to do so. To prevent this we have created a tool to powerset test feature combinations. You can run `just powerset <command>` to run the powerset tests locally. We run these tests only when attempting to merge a PR via `?brawl merge` or `?brawl try`

#### Deny

When adding deps, we need to make sure their licenses are allowed, you can run `just deny` to check the licenses of the deps.

#### Docs

We have a ci job that will check that the docs are built correctly, you can run `just docs` to build the docs locally. You can preview the docs by running `just docs-serve`.

#### Tests

We have a ci job that will check that the tests are passing, you can run `just test` to run the tests locally.

##### Coverage

You can also see the coverage of the tests generated by the command by either previewing the `lcov.info` file or by running `just coverage-serve` to serve the coverage report.

##### Valgrind

We use valgrind to check for memory leaks in our tests. You can run `just valgrind` to run the tests with valgrind. Some tests are disabled because they are tests based on timings and valgrind runs them much too slow. If you add a test that is based on timings, you should disable it with valgrind `#[cfg(not(valgrind))]`.

###### Updating the `valgrind_supressions.log` file

If you add some new tests that leak some memory into the global state you need to update the `valgrind_supressions.log` file in order to keep the CI green.

After you run `just grind` you will see an output of all of the failed tests along with a valgrind output like such

```
──── STDERR:             scuffle-metrics collector::tests::test_metric_with_string_attribute
==551264== Memcheck, a memory error detector
==551264== Copyright (C) 2002-2024, and GNU GPL'd, by Julian Seward et al.
==551264== Using Valgrind-3.24.0 and LibVEX; rerun with -h for copyright info
==551264== Command: /home/simao/Projects/rust/scuffle/target/debug/deps/scuffle_metrics-44942b38869cd765 --exact collector::tests::test_metric_with_string_attribute --nocapture
==551264== 
==551264== 
==551264== HEAP SUMMARY:
==551264==     in use at exit: 139,300 bytes in 32 blocks
==551264==   total heap usage: 718 allocs, 686 frees, 230,102 bytes allocated
==551264== 
==551264== 24 bytes in 1 blocks are possibly lost in loss record 4 of 28
==551264==    at 0x48457A8: malloc (vg_replace_malloc.c:446)
==551264==    by 0x61A6F1: alloc::alloc::alloc (alloc.rs:96)
==551264==    by 0x61A7F9: alloc::alloc::Global::alloc_impl (alloc.rs:192)
==551264==    by 0x61A178: allocate (alloc.rs:254)
==551264==    by 0x61A178: alloc::sync::Arc<[T]>::allocate_for_slice::{{closure}} (sync.rs:2039)
==551264==    by 0x619E6E: alloc::sync::Arc<T>::allocate_for_layout (sync.rs:1952)
==551264==    by 0x61A121: alloc::sync::Arc<[T]>::allocate_for_slice (sync.rs:2037)
==551264==    by 0x619FD2: alloc::sync::Arc<[T]>::copy_from_slice (sync.rs:2051)
==551264==    by 0x35C9C0: from_slice<u8> (sync.rs:2142)
==551264==    by 0x35C9C0: from<u8> (sync.rs:3631)
==551264==    by 0x35C9C0: <alloc::sync::Arc<str> as core::convert::From<&str>>::from (sync.rs:3669)
==551264==    by 0x385E61: scuffle_metrics::collector::tests::test_metric_with_string_attribute::example::request_with_method (collector.rs:608)
==551264==    by 0x38555C: scuffle_metrics::collector::tests::test_metric_with_string_attribute (collector.rs:624)
==551264==    by 0x37E3D6: scuffle_metrics::collector::tests::test_metric_with_string_attribute::{{closure}} (collector.rs:607)
==551264==    by 0x359A25: core::ops::function::FnOnce::call_once (function.rs:250)
==551264== 
{
   <insert_a_suppression_name_here>
   Memcheck:Leak
   match-leak-kinds: possible
   fun:malloc
   fun:_ZN5alloc5alloc5alloc17h9107b1cc2a0aa61aE
   fun:_ZN5alloc5alloc6Global10alloc_impl17hd5a77936c3f57790E
   fun:allocate
   fun:_ZN5alloc4sync22Arc$LT$$u5b$T$u5d$$GT$18allocate_for_slice28_$u7b$$u7b$closure$u7d$$u7d$17hfb887c7502b0a26dE
   fun:_ZN5alloc4sync12Arc$LT$T$GT$19allocate_for_layout17hd0c84398dc717102E
   fun:_ZN5alloc4sync22Arc$LT$$u5b$T$u5d$$GT$18allocate_for_slice17h390937e173d2416aE
   fun:_ZN5alloc4sync22Arc$LT$$u5b$T$u5d$$GT$15copy_from_slice17h846493007d1e92a5E
   fun:from_slice<u8>
   fun:from<u8>
   fun:_ZN82_$LT$alloc..sync..Arc$LT$str$GT$$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h453e33cc9264d222E
   fun:_ZN15scuffle_metrics9collector5tests33test_metric_with_string_attribute7example19request_with_method17ha517184047e9b9a5E
   fun:_ZN15scuffle_metrics9collector5tests33test_metric_with_string_attribute17hc35b598541e835dfE
   fun:_ZN15scuffle_metrics9collector5tests33test_metric_with_string_attribute28_$u7b$$u7b$closure$u7d$$u7d$17h39e89abbe5c8ab7eE
   fun:_ZN4core3ops8function6FnOnce9call_once17h64835dc5cce34b93E
}
```

At the end of the output there is a section that starts with `{` and ends with `}`. This is the suppression that you need to add to the `valgrind_supressions.log` file. We typically clean up this supression a bit by removing unneeded function calls.

In the above it would be the `fun:_ZN15scuffle_metrics9collector5tests33test_metric_with_string_attribute17hc35b598541e835dfE` line as that maps to `scuffle_metrics::collector::tests::test_metric_with_string_attribute (collector.rs:624)` in the code.

We must remove the end hash (`17hc35b598541e835dfE`) and replace it with a `*`

So it looks like `fun:_ZN15scuffle_metrics9collector5tests33test_metric_with_string_attribute*`

Then we can remove all other lines and replace it with a `...`

So the final supression comes out to

```
{
   <insert_a_suppression_name_here>
   Memcheck:Leak
   match-leak-kinds: possible
   ...
   fun:_ZN15scuffle_metrics9collector5tests33test_metric_with_string_attribute*
   ...
}
```

We then add a useful description for why we are supressing this leak, and then we add it to the `valgrind_supressions.log` file and run `just grind` again to make sure the leak is fixed.

#### Hakari

Hackari is a way to improve build times when using workspaces. If you make any dependency changes, you should run `just workspace-hack` to update the hackari cache.

### Merging

We use a custom bot named [brawl](https://github.com/scufflecloud/brawl) to merge pull requests. When a PR has been approved by a maintainer, we will then do `?brawl merge` to add the PR to the merge queue. The reason we do this is because we want to make sure that the PR is ready to be merged and that it has been tested with changes that were not directly present in the PR. Since we do not require PRs to be rebased before merging we want to make sure that the PR works on the latest `main` branch.

### Release

Releasing crates is done by running a workflow dispatch on the `Create Release PR` workflow with the crate name as the input. This will then create a new PR with the crate's version bumped and the changelog updated.

## Questions

If you have any questions, please ask in the [discord server](https://discord.gg/scuffle) or create an issue on the repo or in the discussion section

Please do not hesitate to ask questions; we are here to help you and make sure you are comfortable contributing to this project. If you need help following the design documents or need clarification about the codebase, please ask us, and we will help you.

## Thank you

Thank you for taking the time to read this document and for contributing to this project. We are very excited to have you on board, and we hope you enjoy your time here.
