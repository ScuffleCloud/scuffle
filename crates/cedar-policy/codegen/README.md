<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-cedar-policy-codegen
<!-- sync-readme ]] -->

> [!WARNING]
> This crate is under active development and may not be stable.

<!-- sync-readme badge -->

---

<!-- sync-readme rustdoc [[ -->
Cedar is a policy language used to express permisisons using a relationship model.

This crate extends the [`cedar-policy`](https://docs.rs/cedar-policy) crate by adding code generator for cedar schemas.

You can then use this in combo with cedar to have type-safe schema evaluation.

### Example

````rust
let schema = std::fs::read_to_string("./static.cedarschema").expect("failed to read");

let config = scuffle_cedar_policy_codegen::Config::default()
    .generate_from_schema(&schema)
    .expect("valid schema");

let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("no such env")).join("generated.rs");
std::fs::write(output, config.to_string()).expect("failed to write output");
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
