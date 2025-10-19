use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

use lean_macro_internals::parse;

pub fn impl_modules(input: TokenStream2) -> syn::Result<TokenStream2> {
    let derive_input: DeriveInput = syn::parse2(input)?;
    let name = &derive_input.ident;
    let module_initialization_function_ident =
        parse::parse_lean_module_initialization_function_from_rust_module_initializer_type_name(
            name,
        )
        .map_err(|mut error| {
            error.combine(syn::Error::new(
                name.span(),
                "error using `Modules` trait with `#[derive]`",
            ));
            error
        })?;

    let generated = quote! {
        unsafe impl lean::Modules for #name {
            unsafe fn initialize_modules(builtin: u8, lean_io_world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res {
                unsafe { #module_initialization_function_ident(builtin, lean_io_world) }
            }
        }
    };
    Ok(generated)
}
