use syn::parse_macro_input;

mod combine_lean_module_initializers;
mod create_module_trait;

use combine_lean_module_initializers::CombineLeanModuleInitializers;

#[proc_macro_attribute]
pub fn create_module_trait(
    input: proc_macro::TokenStream,
    annotated_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let output = create_module_trait::impl_create_module_trait(input.into(), annotated_item.into());

    output.unwrap_or_else(syn::Error::into_compile_error).into()
}

#[proc_macro]
pub fn combine_lean_module_initializers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = parse_macro_input!(input as CombineLeanModuleInitializers);
    let expanded = combine_lean_module_initializers::generate(parsed_input);

    proc_macro::TokenStream::from(expanded)
}
