use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, LitBool, Token};

struct ModuleDefinition {
    name: Ident,
    info: bool,
    warn: bool,
    error: bool,
    debug: bool,
}

impl syn::parse::Parse for ModuleDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let info_kw: Ident = input.parse()?;
        if info_kw != "info" {
            return Err(syn::Error::new(info_kw.span(), "expected `info`"));
        }
        input.parse::<Token![=]>()?;
        let info: LitBool = input.parse()?;
        input.parse::<Token![,]>()?;

        let warn_kw: Ident = input.parse()?;
        if warn_kw != "warn" {
            return Err(syn::Error::new(warn_kw.span(), "expected `warn`"));
        }
        input.parse::<Token![=]>()?;
        let warn: LitBool = input.parse()?;
        input.parse::<Token![,]>()?;

        let error_kw: Ident = input.parse()?;
        if error_kw != "error" {
            return Err(syn::Error::new(error_kw.span(), "expected `error`"));
        }
        input.parse::<Token![=]>()?;
        let error: LitBool = input.parse()?;
        input.parse::<Token![,]>()?;

        let debug_kw: Ident = input.parse()?;
        if debug_kw != "debug" {
            return Err(syn::Error::new(debug_kw.span(), "expected `debug`"));
        }
        input.parse::<Token![=]>()?;
        let debug: LitBool = input.parse()?;

        Ok(ModuleDefinition {
            name,
            info: info.value,
            warn: warn.value,
            error: error.value,
            debug: debug.value,
        })
    }
}

#[proc_macro]
pub fn define_module(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as ModuleDefinition);
    let name = &def.name;
    let name_lower = name.to_string().to_lowercase();

    let info_val = def.info;
    let warn_val = def.warn;
    let error_val = def.error;
    let debug_val = def.debug;

    let info_macro = Ident::new(&format!("{}_info", name_lower), name.span());
    let warn_macro = Ident::new(&format!("{}_warn", name_lower), name.span());
    let error_macro = Ident::new(&format!("{}_error", name_lower), name.span());
    let debug_macro = Ident::new(&format!("{}_debug", name_lower), name.span());

    let expanded = quote! {
        pub struct #name;

        impl m_log::ModuleLog for #name {
            const INFO: bool = #info_val;
            const WARN: bool = #warn_val;
            const ERROR: bool = #error_val;
            const DEBUG: bool = #debug_val;
        }

        macro_rules! #info_macro {
            ($($arg:tt)*) => {
                m_info!(#name, $($arg)*)
            };
        }

        macro_rules! #warn_macro {
            ($($arg:tt)*) => {
                m_warn!(#name, $($arg)*)
            };
        }

        macro_rules! #error_macro {
            ($($arg:tt)*) => {
                m_error!(#name, $($arg)*)
            };
        }

        macro_rules! #debug_macro {
            ($($arg:tt)*) => {
                m_debug!(#name, $($arg)*)
            };
        }
    };

    expanded.into()
}