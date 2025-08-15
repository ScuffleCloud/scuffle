# Scuffle Contribution Guide

## Code of Conduct

We have a [Code of Conduct](./CODE_OF_CONDUCT.md) that we expect all contributors to follow. Please read it before contributing.

## Developer Environment

### Bazel

All developers need to have [bazelisk](https://github.com/bazelbuild/bazelisk) installed, you can find the installation instructions [here](https://bazel.build/install/bazelisk).

We use bazel instead of cargo to build the project. This is because cargo struggles to cache builds for large projects and we often ended up having cache invalidation issues. Bazel also allows us to have a more consistent build environment across different machines and CI and support for languages other than rust.

One example of how we optimize our cache use is external dependencies:

We vendor all our dependencies [`just vendor`](#local-commnads), and have them built in `opt` (release) mode. This means that we do not need to rebuild dependencies when switching between debug and release builds, and also we don't instrument coverage. Meaning we can use the same cached builds for debug, release, coverage, valgrind, clippy, rust-analyzer builds.

#### Scripts / Tools

We provide a bunch of tools which we vendor by wrapping them into a bazel rule, you can find them in [`tools/scripts`](./tools/scripts/README.md). We recommend adding this directory to the front of your `PATH` (automatically done if you use [direnv](#environment-variables)).

### VSCode Setup

If you use vscode you can setup rust-analyzer to work by adding the following to your settings.json:

```json
{
    "rust-analyzer.server.path": "${workspaceFolder}/build/utils/rust/analyzer/lsp.sh",
    "rust-analyzer.workspace.discoverConfig": {
        "command": [
            "${workspaceFolder}/build/utils/rust/analyzer/discover.sh"
        ],
        "progressLabel": "rust_analyzer",
        "filesToWatch": [
            "BUILD",
            "BUILD.bazel",
            "MODULE.bazel"
        ]
    },
    "rust-analyzer.check.overrideCommand": [
        "${workspaceFolder}/build/utils/rust/analyzer/check.sh"
    ],
}
```

for a bazel lsp you can use the bazel extension for vscode and download [starpls](https://github.com/withered-magic/starpls), adding the following to your settings.json:

```json
{
   "bazel.lsp.command": "starpls",
   "bazel.lsp.args": [
      "server",
      "--experimental_infer_ctx_attributes",
      "--experimental_enable_label_completions",
      "--experimental_use_code_flow_analysis",
      "--bazel_path=bazelisk",
   ],
   "bazel.executable": "bazelisk",
   "bazel.enableCodeLens": true
}
```

### Environment Variables

We advice you to use [direnv](https://direnv.net/) to load the .envrc file, which sets up a few environment variables needed for development.

### Nix

We have a [nix shell setup](nix/README.md) too, (automatically loaded if you use direnv).

## Local Commnads

| Command | Description |
|---------|-------------|
| `just test` | Run all tests |
| `just grind` | Run tests with valgrind |
| `just lint` | Lint the code & try auto-fix linting errors |
| `just fmt` | Format the code |
| `just deny` | Check that all dependencies have allowed licenses |
| `just docs` | Build the docs |
| `just docs-serve` | Serve the docs locally |
| `just vendor` | Vendor the dependencies |

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

### Merging

We use a custom bot named [brawl](https://github.com/scufflecloud/brawl) to merge pull requests. When a PR has been approved by a maintainer, we will then do `?brawl merge` to add the PR to the merge queue. The reason we do this is because we want to make sure that the PR is ready to be merged and that it has been tested with changes that were not directly present in the PR. Since we do not require PRs to be rebased before merging we want to make sure that the PR works on the latest `main` branch.

### Release

Releasing crates is done by running a workflow dispatch on the `Create Release PR` workflow with the crate name as the input. This will then create a new PR with the crate's version bumped and the changelog updated.

## Questions

If you have any questions, please ask in the [discord server](https://discord.gg/scuffle) or create an issue on the repo or in the discussion section

Please do not hesitate to ask questions; we are here to help you and make sure you are comfortable contributing to this project. If you need help following the design documents or need clarification about the codebase, please ask us, and we will help you.

## Thank you

Thank you for taking the time to read this document and for contributing to this project. We are very excited to have you on board, and we hope you enjoy your time here.
