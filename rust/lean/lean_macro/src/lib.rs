mod module;

#[proc_macro_attribute]
pub fn create_module_trait(
    input: proc_macro::TokenStream,
    annotated_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let output = module::impl_create_module_trait(input.into(), annotated_item.into());

    output.unwrap_or_else(syn::Error::into_compile_error).into()
}
