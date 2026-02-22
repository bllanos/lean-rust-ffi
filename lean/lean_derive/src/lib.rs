mod lean_external_type_tag;
mod module;

/// Derive `LeanExternalTypeTag` assuming the type does not contain internal
/// Lean objects
#[proc_macro_derive(LeanExternalTypeTag)]
pub fn lean_external_type_tag_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = lean_external_type_tag::impl_lean_external_type_tag(input.into());

    output.unwrap_or_else(syn::Error::into_compile_error).into()
}

#[proc_macro_derive(Modules, attributes(initializer))]
pub fn modules_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = module::impl_modules(input.into());

    output.unwrap_or_else(syn::Error::into_compile_error).into()
}
