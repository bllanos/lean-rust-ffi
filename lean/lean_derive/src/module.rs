use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Path, spanned::Spanned};

pub fn impl_modules(input: TokenStream2) -> syn::Result<TokenStream2> {
    let derive_input: DeriveInput = syn::parse2(input)?;
    let name = &derive_input.ident;
    let module_initialization_function_path =
        parse_initializer_path(&derive_input).map_err(|mut error| {
            error.combine(syn::Error::new(
                name.span(),
                "error using `Modules` trait with `#[derive]`",
            ));
            error
        })?;

    let generated = quote! {
        unsafe impl ::lean::Modules for #name {
            unsafe fn initialize_modules(builtin: u8) -> ::lean_sys::lean_obj_res {
                unsafe { #module_initialization_function_path(builtin) }
            }
        }
    };
    Ok(generated)
}

const INITIALIZER_ATTRIBUTE: &str = "initializer";

fn parse_initializer_path(input: &DeriveInput) -> syn::Result<Path> {
    let attribute = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(INITIALIZER_ATTRIBUTE))
        .ok_or_else(|| {
            syn::Error::new(
                input.span(),
                format!("expected an `{INITIALIZER_ATTRIBUTE}` attribute"),
            )
        })?;
    let argument: Path = attribute.parse_args().map_err(|mut error| {
        error.combine(syn::Error::new(
            attribute.span(),
            format!("expected `{INITIALIZER_ATTRIBUTE}` attribute to contain a path to an initialization function"),
        ));
        error
    })?;
    Ok(argument)
}
