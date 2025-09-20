use proc_macro::TokenStream;

mod compile_schema;

#[proc_macro]
pub fn compile_schema(input: TokenStream) -> TokenStream {
    match compile_schema::compile_schema(input.into()) {
        Ok(token) => token.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
