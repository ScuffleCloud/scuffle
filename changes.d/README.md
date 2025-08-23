# Change Log Fragments

This directory contains the changelog fragments for the Scuffle project.

Each file in this directory corresponds to a pull request.

Files should be named like `pr-<pr-number>.toml`.

The content of the file should be a package name followed by a list of changes.

Example:

```toml
[[scuffle-rtmp]]
category = "bug"
description = "Fixed a bug"

[[scuffle-rtmp]]
breaking = true
category = "feat"
description = "Added a new feature"
authors = ["@TroyKomodo"]
```

This will result in 2 changelog entries being added to the `scuffle-rtmp` package.
The first entry will have the category `bug` and the description `Fixed a bug`.
The second entry will be a breaking change and have the category `feat` and the description `Added a new feature`.

The categories `bug` and `feat` do not have any special meaning. Other than, all changes of the same
category will be grouped together in the changelog. Breaking changes will be in their own sub group.

The `breaking` field is optional. If it is not present, the change is not considered a breaking change.

The `authors` field is optional. If it is not present, the change will not have any authors.

You can refer to the [example file](./example.toml) for an example of how to write a changelog fragment.
