use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::utils::SetSpan;

#[derive(darling::FromMeta)]
struct Args {
    name: syn::LitStr,
    version: syn::Expr,
    methods: DataTypeMethods,
}

enum SetOrPath {
    Set(syn::Path),
    Path(syn::Path),
}

impl SetOrPath {
    fn path(&self, name: &syn::Ident, generics: &syn::Generics) -> syn::Path {
        match self {
            Self::Set(func) => {
                let ty = generics.type_params();
                let path: syn::Path = syn::parse_quote! { #name::<#(#ty),*>::#func };
                path.set_span(func.span())
            }
            Self::Path(path) => path.clone(),
        }
    }
}

impl darling::FromMeta for SetOrPath {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            syn::Meta::Path(path) => Ok(Self::Set(path.clone())),
            syn::Meta::NameValue(value) => {
                let expr = &value.value;
                Ok(Self::Path(syn::parse_quote!(#expr)))
            }
            _ => todo!(),
        }
    }
}

#[derive(darling::FromMeta)]
struct DataTypeMethods {
    rdb_load: SetOrPath,
    rdb_save: SetOrPath,
    aof_rewrite: Option<SetOrPath>,
    mem_usage: Option<SetOrPath>,
    digest: Option<SetOrPath>,
    aux_load: Option<SetOrPath>,
    aux_save: Option<SetOrPath>,
    free_effort: Option<SetOrPath>,
    unlink: Option<SetOrPath>,
    copy: Option<SetOrPath>,
    defrag: Option<SetOrPath>,
    mem_usage2: Option<SetOrPath>,
    free_effort2: Option<SetOrPath>,
    unlink2: Option<SetOrPath>,
    copy2: Option<SetOrPath>,
    aux_save2: Option<SetOrPath>,
}

macro_rules! func_to_tokens {
    (
        $fn_ident:ident
        $trait_ident:ident
        $fn_ty:ty
    ) => {
        fn $fn_ident(&self, name: &syn::Ident, generics: &syn::Generics) -> TokenStream {
            func_to_tokens!(@body; name, generics, self.$fn_ident, $trait_ident, $fn_ty)
        }
    };
    (
        opt $fn_ident:ident
        $trait_ident:ident
        $fn_ty:ty
    ) => {
        fn $fn_ident(&self, name: &syn::Ident, generics: &syn::Generics) -> TokenStream {
            match &self.$fn_ident {
                Some(value) => func_to_tokens!(@body; name, generics, value, $trait_ident, $fn_ty),
                None => quote::quote!(::core::option::Option::None)
            }
        }
    };
    (
        @body; $name:ident, $generics:ident, $path:expr, $trait_ident:ident, $fn_ty:ty
    ) => {{
        let (imp, ty, wh) = $generics.split_for_impl();
        let path = $path.path($name, $generics);
        quote::quote!({
            struct Tmp;
            impl #imp redis_module_ext::data_type::$trait_ident<Tmp> for #$name #ty #wh {
                const FN: redis_module_ext::data_type::$fn_ty = #path;
            }

            <#$name #ty as redis_module_ext::data_type::$trait_ident<Tmp>>::extern_fn()
        })
    }}
}

impl DataTypeMethods {
    func_to_tokens!(rdb_load RdbLoad RdbLoadFn<Self>);

    func_to_tokens!(rdb_save RdbSave RdbSaveFn<Self>);

    func_to_tokens!(opt aof_rewrite AofRewrite AofRewriteFn<Self>);

    func_to_tokens!(opt mem_usage MemUsage MemUsageFn<Self>);

    func_to_tokens!(opt digest Digest DigestFn<Self>);

    func_to_tokens!(opt aux_load AuxLoad AuxLoadFn);

    func_to_tokens!(opt aux_save AuxSave AuxSaveFn);

    func_to_tokens!(opt free_effort FreeEffort FreeEffortFn<Self>);

    func_to_tokens!(opt unlink Unlink UnlinkFn<Self>);

    func_to_tokens!(opt copy Copy CopyFn<Self>);

    func_to_tokens!(opt defrag Defrag DefragFn<Self>);

    func_to_tokens!(opt mem_usage2 MemUsage2 MemUsage2Fn<Self>);

    func_to_tokens!(opt free_effort2 FreeEffort2 FreeEffort2Fn<Self>);

    func_to_tokens!(opt unlink2 Unlink2 Unlink2Fn<Self>);

    func_to_tokens!(opt copy2 Copy2 Copy2Fn<Self>);

    func_to_tokens!(opt aux_save2 AuxSave2 AuxSave2Fn);
}

pub fn macro_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attr)?;

    let args = Args::from_list(&attr_args)?;

    if args.name.value().len() != 9 {
        return Err(syn::Error::new(
            args.name.span(),
            format!("names must be exactly 9 characters long, found: {}", args.name.value().len()),
        ));
    }

    let item = syn::parse2::<syn::Item>(item)?;

    let (name, generics) = match &item {
        syn::Item::Struct(item) => (&item.ident, &item.generics),
        syn::Item::Enum(item) => (&item.ident, &item.generics),
        syn::Item::Type(item) => (&item.ident, &item.generics),
        syn::Item::Union(item) => (&item.ident, &item.generics),
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "data type can only be made from a struct / enum / type / union",
            ));
        }
    };

    let type_name = &args.name;
    let type_version = &args.version;

    let rdb_load = args.methods.rdb_load(name, generics);
    let rdb_save = args.methods.rdb_save(name, generics);
    let aof_rewrite = args.methods.aof_rewrite(name, generics);
    let mem_usage = args.methods.mem_usage(name, generics);
    let digest = args.methods.digest(name, generics);
    let aux_load = args.methods.aux_load(name, generics);
    let aux_save = args.methods.aux_save(name, generics);
    let free_effort = args.methods.free_effort(name, generics);
    let unlink = args.methods.unlink(name, generics);
    let copy = args.methods.copy(name, generics);
    let defrag = args.methods.defrag(name, generics);
    let mem_usage2 = args.methods.mem_usage2(name, generics);
    let free_effort2 = args.methods.free_effort2(name, generics);
    let unlink2 = args.methods.unlink2(name, generics);
    let copy2 = args.methods.copy2(name, generics);
    let aux_save2 = args.methods.aux_save2(name, generics);

    let (imp, ty, wh) = generics.split_for_impl();

    Ok(quote::quote! {
        #item

        const _: () = {
            impl #imp redis_module_ext::data_type::RedisDataType for #name #ty #wh {
                const NAME: &'static str = #type_name;
                const VERSION: i32 = #type_version;

                fn module_methods(_: &::redis_module_ext::redis::Context) -> redis_module_ext::raw::RedisModuleTypeMethods {
                    redis_module_ext::raw::RedisModuleTypeMethods {
                        version: redis_module_ext::raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
                        aux_save_triggers: 0,
                        rdb_load: #rdb_load,
                        rdb_save: #rdb_save,
                        aof_rewrite: #aof_rewrite,
                        mem_usage: #mem_usage,
                        digest: #digest,
                        free: {
                            unsafe extern "C" fn free<T>(value: *mut ::std::os::raw::c_void) {
                                drop(unsafe { ::std::boxed::Box::from_raw(value.cast::<T>()) });
                            }

                            Some(free::<#name #ty>)
                        },
                        aux_load: #aux_load,
                        aux_save: #aux_save,
                        free_effort: #free_effort,
                        unlink: #unlink,
                        copy: #copy,
                        defrag: #defrag,
                        mem_usage2: #mem_usage2,
                        free_effort2: #free_effort2,
                        unlink2: #unlink2,
                        copy2: #copy2,
                        aux_save2: #aux_save2,
                    }
                }
            }
        };
    })
}
