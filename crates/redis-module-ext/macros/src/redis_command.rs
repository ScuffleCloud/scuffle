use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::{Span, TokenStream};

use crate::utils::{collect_to_vec, format_command_ident, opt_to_tokens, repeated_parse, str_to_cstr};

#[derive(darling::FromMeta, Debug)]
struct Args {
    name: syn::LitStr,
    #[darling(default)]
    summary: Option<syn::LitStr>,
    #[darling(default)]
    complexity: Option<syn::LitStr>,
    #[darling(default)]
    since: Option<syn::LitStr>,
    #[darling(default, multiple)]
    history: Vec<History>,
    #[darling(default)]
    tips: Option<syn::LitStr>,
    #[darling(default)]
    arity: Option<syn::Expr>,
    #[darling(default, with = repeated_parse)]
    flags: Vec<syn::Ident>,
    #[darling(default, with = repeated_parse)]
    enterprise_flags: Vec<syn::Ident>,
    #[darling(default, multiple)]
    key_spec: Vec<KeySpec>,
    #[darling(default, multiple)]
    arg: Vec<CommandArg>,
}

#[derive(darling::FromMeta, Debug)]
struct History {
    #[darling(default)]
    since: Option<syn::LitStr>,
    #[darling(default)]
    changes: Option<syn::LitStr>,
}

#[derive(darling::FromMeta, Debug)]
struct KeySpec {
    #[darling(default)]
    notes: Option<syn::LitStr>,
    #[darling(default, with = repeated_parse)]
    flags: Vec<syn::Ident>,
    begin_search: BeginSearch,
    #[darling(default)]
    find_keys: Option<FindKeys>,
}

#[derive(darling::FromMeta, Debug)]
struct CommandArg {
    name: syn::LitStr,
    #[darling(default)]
    kind: Option<syn::Ident>,
    #[darling(default)]
    key_spec_index: Option<syn::LitInt>,
    #[darling(default)]
    token: Option<syn::LitStr>,
    #[darling(default)]
    summary: Option<syn::LitStr>,
    #[darling(default)]
    since: Option<syn::LitStr>,
    #[darling(default)]
    flags: Option<syn::LitInt>,
    #[darling(default)]
    deprecated_since: Option<syn::LitStr>,
    #[darling(default, multiple)]
    arg: Vec<CommandArg>,
    #[darling(default)]
    display_text: Option<syn::LitStr>,
}

#[derive(darling::FromMeta, Debug)]
enum BeginSearch {
    Index(i32),
    Keyword(BeginSearchKeyword),
}

#[derive(darling::FromMeta, Debug)]
struct BeginSearchKeyword {
    keyword: syn::LitStr,
    start_from: i32,
}

#[derive(darling::FromMeta, Debug)]
enum FindKeys {
    Range(FindKeysRange),
    Keynum(FindKeysKeynum),
}

#[derive(darling::FromMeta, Debug)]
struct FindKeysRange {
    last_key: i32,
    steps: i32,
    limit: i32,
}

#[derive(darling::FromMeta, Debug)]
struct FindKeysKeynum {
    key_num_idx: i32,
    first_key: i32,
    key_step: i32,
}

pub fn macro_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attr)?;

    let macro_args = Args::from_list(&attr_args)?;

    let item: syn::ItemFn = syn::parse2(item)?;

    let mut struct_ident = format_command_ident(&item.sig.ident);
    struct_ident.set_span(Span::call_site());
    let vis = &item.vis;
    let (imp, ty, wh) = item.sig.generics.split_for_impl();

    let name_cstr = str_to_cstr(&macro_args.name)?;
    let summary = opt_to_tokens(macro_args.summary.as_ref().map(str_to_cstr).transpose()?);
    let complexity = opt_to_tokens(macro_args.complexity.as_ref().map(str_to_cstr).transpose()?);
    let since = opt_to_tokens(macro_args.since.as_ref().map(str_to_cstr).transpose()?);
    let tips = opt_to_tokens(macro_args.tips.as_ref().map(str_to_cstr).transpose()?);
    let arity = macro_args.arity.unwrap_or_else(|| syn::parse_quote!(-1));
    let history = macro_args
        .history
        .iter()
        .map(|history| {
            let since = opt_to_tokens(history.since.as_ref().map(str_to_cstr).transpose()?);
            let changes = opt_to_tokens(history.changes.as_ref().map(str_to_cstr).transpose()?);
            Ok(quote::quote! {
                ::redis_module_ext::command::RedisModuleCommandHistoryEntry {
                    since: #since.map(::std::borrow::Cow::Borrowed),
                    changes: #changes.map(::std::borrow::Cow::Borrowed),
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let key_specs = macro_args
        .key_spec
        .iter()
        .map(|key_spec| {
            let notes = opt_to_tokens(key_spec.notes.as_ref().map(str_to_cstr).transpose()?);
            let begin_search = match &key_spec.begin_search {
                BeginSearch::Index(idx) => quote::quote!(::redis_module_ext::command::KeySpecBeginSearch::Index(#idx)),
                BeginSearch::Keyword(BeginSearchKeyword { keyword, start_from }) => {
                    let keyword = str_to_cstr(keyword)?;
                    quote::quote!(::redis_module_ext::command::KeySpecBeginSearch::Keyword {
                        keyword: ::std::borrow::Cow::Borrowed(#keyword.into()),
                        start_from: #start_from,
                    })
                }
            };
            let find_keys = match &key_spec.find_keys {
                None => quote::quote!(::core::option::Option::None),
                Some(FindKeys::Keynum(FindKeysKeynum {
                    first_key,
                    key_num_idx,
                    key_step,
                })) => quote::quote! {
                    ::core::option::Option::Some(::redis_module_ext::command::KeySpecFindKeys::Keynum {
                        keynum_idx: #key_num_idx,
                        first_key: #first_key,
                        key_step: #key_step,
                    })
                },
                Some(FindKeys::Range(FindKeysRange { last_key, steps, limit })) => quote::quote! {
                    ::core::option::Option::Some(::redis_module_ext::command::KeySpecFindKeys::Range {
                        last_key: #last_key,
                        key_step: #steps,
                        limit: #limit,
                    })
                },
            };

            let flags = collect_to_vec(
                key_spec
                    .flags
                    .iter()
                    .map(|ident| quote::quote!(redis_module_ext::command::KeySpecFlag::#ident)),
            );

            Ok(quote::quote! {
                redis_module_ext::command::RedisModuleCommandKeySpec {
                    notes: #notes.map(::std::borrow::Cow::Borrowed),
                    flags: #flags,
                    begin_search: #begin_search,
                    find_keys: #find_keys,
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    fn convert_arg(arg: &CommandArg) -> syn::Result<TokenStream> {
        let neg_1 = syn::LitInt::new("-1", Span::call_site());
        let zero = syn::LitInt::new("0", Span::call_site());
        let name = str_to_cstr(&arg.name)?;
        let kind = arg.kind.clone().unwrap_or_else(|| quote::format_ident!("String"));
        let key_spec_idx = arg.key_spec_index.as_ref().unwrap_or(&neg_1);
        let token = opt_to_tokens(arg.token.as_ref().map(str_to_cstr).transpose()?);
        let summary = opt_to_tokens(arg.summary.as_ref().map(str_to_cstr).transpose()?);
        let since = opt_to_tokens(arg.since.as_ref().map(str_to_cstr).transpose()?);
        let deprecated_since = opt_to_tokens(arg.deprecated_since.as_ref().map(str_to_cstr).transpose()?);
        let display_text = opt_to_tokens(arg.display_text.as_ref().map(str_to_cstr).transpose()?);
        let flags = arg.flags.as_ref().unwrap_or(&zero);
        let sub_args = arg.arg.iter().map(convert_arg).collect::<syn::Result<Vec<_>>>()?;
        let sub_args = collect_to_vec(sub_args);

        Ok(quote::quote! {{
            ::redis_module_ext::command::RedisModuleCommandArg {
                name: ::std::borrow::Cow::Borrowed((#name)),
                kind: ::redis_module_ext::command::RedisModuleCommandArgKind::#kind,
                key_spec_idx: (#key_spec_idx),
                token: (#token).map(::std::borrow::Cow::Borrowed),
                summary: (#summary).map(::std::borrow::Cow::Borrowed),
                since: (#since).map(::std::borrow::Cow::Borrowed),
                flags: (#flags),
                deprecated_since: (#deprecated_since).map(::std::borrow::Cow::Borrowed),
                sub_args: #sub_args,
                display_text: (#display_text).map(::std::borrow::Cow::Borrowed),
            }
        }})
    }

    let args = macro_args.arg.iter().map(convert_arg).collect::<syn::Result<Vec<_>>>()?;

    let fn_generics = item.sig.generics.params.iter().filter_map(|param| match param {
        syn::GenericParam::Const(c) => Some(&c.ident),
        syn::GenericParam::Type(ty) => Some(&ty.ident),
        _ => None,
    });

    let marker_ty = item
        .sig
        .generics
        .lifetimes()
        .map(|lt| {
            let lt = &lt.lifetime;
            quote::quote!(&#lt ())
        })
        .chain(item.sig.generics.type_params().map(|ty| {
            let ident = &ty.ident;
            quote::quote!(#ident)
        }));

    let history = collect_to_vec(history);
    let key_specs = collect_to_vec(key_specs);
    let args = collect_to_vec(args);

    let syn::Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        fn_token,
        ident,
        inputs,
        variadic,
        output,
        ..
    } = &item.sig;

    let inputs = inputs.pairs();

    let flags = collect_to_vec(macro_args.flags.iter().map(|ident| {
        quote::quote! {
            ::redis_module_ext::command::CommandFlag::#ident
        }
    }));

    let enterprise_flags = if !macro_args.enterprise_flags.is_empty() {
        let enterprise_flags = macro_args.enterprise_flags.iter().map(|ident| {
            quote::quote! {
                ::redis_module_ext::command::CommandFlag::#ident
            }
        });

        quote::quote! {
            if ctx.is_enterprise() {
                flags.extend([#(#enterprise_flags),*]);
            }
        }
    } else {
        quote::quote!()
    };

    Ok(quote::quote! {
        #item

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #struct_ident #imp #wh {
            marker: ::core::marker::PhantomData<(#(#marker_ty),*)>,
        }

        impl #imp ::redis_module_ext::command::RedisCommand for #struct_ident #ty #wh {
            const NAME: &'static ::std::ffi::CStr = #name_cstr;

            fn flags(ctx: &::redis_module_ext::redis::Context) -> Vec<::redis_module_ext::command::CommandFlag> {
                let mut flags = #flags;
                #enterprise_flags
                flags
            }

            fn command_info(_: &::redis_module_ext::redis::Context) -> ::redis_module_ext::command::RedisModuleCommandInfo {
                ::redis_module_ext::command::RedisModuleCommandInfo {
                    summary: (#summary).map(::std::borrow::Cow::Borrowed),
                    complexity: (#complexity).map(::std::borrow::Cow::Borrowed),
                    since: (#since).map(::std::borrow::Cow::Borrowed),
                    history: #history,
                    tips: (#tips).map(::std::borrow::Cow::Borrowed),
                    arity: (#arity),
                    key_specs: #key_specs,
                    args: #args,
                }
            }

            #[allow(unused_mut)]
            #constness #asyncness #unsafety #abi #fn_token invoke(#(#inputs)*, #variadic) #output {
                #ident :: <#(#fn_generics),*> (ctx, args)
            }
        }
    })
}
