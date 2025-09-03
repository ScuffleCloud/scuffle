use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;

use crate::utils::{format_command_ident, repeated_parse, str_to_cstr};

#[derive(darling::FromMeta, Debug)]
struct Args {
    name: syn::LitStr,
    version: syn::LitInt,
    #[darling(default, with = repeated_parse)]
    types: Vec<syn::TypePath>,
    #[darling(default, with = repeated_parse)]
    commands: Vec<syn::TypePath>,
    #[darling(default, with = repeated_parse)]
    merge: Vec<syn::TypePath>,
    #[darling(default)]
    init_fn: Option<syn::Path>,
    #[darling(default)]
    deinit_fn: Option<syn::Path>,
}

pub fn macro_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attr)?;

    let args = Args::from_list(&attr_args)?;

    let item = syn::parse2(item)?;

    let (name, generics) = match &item {
        syn::Item::Struct(item) => (&item.ident, &item.generics),
        syn::Item::Enum(item) => (&item.ident, &item.generics),
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "module can only be implemented on a struct or enum",
            ));
        }
    };

    let (imp, ty, wh) = generics.split_for_impl();

    let mod_version = &args.version;
    let mod_name = str_to_cstr(&args.name)?;

    let data_types = args.types.iter().map(|ty| {
        quote::quote! {
            if <#ty as redis_module_ext::RedisDataType>::register(ctx).is_err() {
                return false;
            }
        }
    });

    let merge_types = &args.merge;

    let commands = args.commands.iter().map(|cmd| {
        let mut cmd_path = cmd.path.clone();
        let mut path = cmd_path.clone();
        let last = path.segments.last_mut().unwrap();
        last.ident = format_command_ident(&last.ident);

        let last = cmd_path.segments.last_mut().unwrap();
        if let syn::PathArguments::AngleBracketed(args) = &mut last.arguments {
            args.colon2_token = Some(Default::default());
            let generics = args
                .args
                .clone()
                .into_pairs()
                .filter(|generic| {
                    matches!(
                        generic.value(),
                        syn::GenericArgument::Const(_) | syn::GenericArgument::Type(_)
                    )
                })
                .collect();
            args.args = generics;
        }

        quote::quote! {
            let _ = #cmd_path;
            <#path as redis_module_ext::RedisCommand>::register(ctx)?;
        }
    });

    let init_fn = args.init_fn.map(|init_fn| quote::quote!(#init_fn(self, ctx)?));
    let deinit_fn = args.deinit_fn.map(|deinit_fn| quote::quote!(#deinit_fn(self, ctx)?));

    Ok(quote::quote! {
        #item

        impl #imp redis_module_ext::module::RedisModule for #name #ty #wh {
            fn name() -> &'static ::std::ffi::CStr {
                #mod_name
            }

            fn version() -> i32 {
                #mod_version
            }

            fn register_data_types(ctx: &::redis_module_ext::redis::Context) -> bool {
                #(#data_types)*
                true #(
                    && #merge_types::register_data_types(ctx)
                )*
            }

            fn register_commands(ctx: &::redis_module_ext::redis::Context) -> ::redis_module_ext::redis::RedisResult<()> {
                #(#commands)*
                #(#merge_types::register_commands(ctx)?;)*
                ::std::result::Result::Ok(())
            }

            fn init_fn(ctx: &::redis_module_ext::redis::Context) -> ::redis_module_ext::redis::RedisResult<()> {
                #init_fn
                #(#merge_types::init_fn(ctx)?;)*
                ::std::result::Result::Ok(())
            }

            fn deinit_fn(ctx: &::redis_module_ext::redis::Context) -> ::redis_module_ext::redis::RedisResult<()> {
                #deinit_fn
                #(#merge_types::deinit_fn(ctx)?;)*
                ::std::result::Result::Ok(())
            }
        }
    })
}
