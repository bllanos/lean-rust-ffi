mod module;

#[proc_macro_derive(Modules)]
pub fn modules_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = module::impl_modules(input.into());

    output.unwrap_or_else(syn::Error::into_compile_error).into()
}
