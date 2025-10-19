use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

use lean_macro_internals::parse;

#[proc_macro_derive(Modules)]
pub fn modules_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input);

    let output = impl_modules(&ast);

    output.unwrap_or_else(syn::Error::into_compile_error).into()
}

fn impl_modules(ast: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let name = &ast.ident;
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
