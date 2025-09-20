// fn main() {
//     const STATIC_POLICIES_SCHEMA_STR: &str = include_str!("../static_policies.cedarschema");

//     let extensions = Extensions::all_available();
//     let (fragment, warnings) =
//         Fragment::from_cedarschema_str(STATIC_POLICIES_SCHEMA_STR, extensions)
//             .expect("Failed to parse Cedar schema");

//     // Print warnings
//     for warning in warnings {
//         println!("warning: {warning}");
//     }

//     // Process the fragment and generate code
//     let file = process_fragment(&fragment).expect("Failed to process Cedar schema fragment");

//     // Write the generated code to file
//     std::fs::write("submod/src/generated.rs", prettyplease::unparse(&file))
//         .expect("Failed to write generated code to file");
// }

use darling::FromMeta;
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct MacroArgs {
    schema: syn::LitStr,
}

pub fn compile_schema(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let span = input.span();
    let args: MacroArgs = syn::parse2(input)?;

    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| syn::Error::new(span, "missing CARGO_MANIFEST_DIR env variable"))?;

    let manifest_dir = std::path::PathBuf::from(manifest_dir);

    let policy_file = manifest_dir.join(args.schema.value());
    let schema_content = std::fs::read_to_string(policy_file)
        .map_err(|err| syn::Error::new(args.schema.span(), format!("failed to read schema file: {err}")))?;

    let output = scuffle_cedar_policy_codegen::Config::new()
        .generate_from_schema(&schema_content)
        .map_err(|err| syn::Error::new(span, format!("failed to generate: {err}")))?;

    Ok(quote::quote!(#output))
}
