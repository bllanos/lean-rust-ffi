use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_derive(Modules)]
pub fn modules_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse macro input");

    impl_modules(&ast)
}

const TYPE_NAME_SUFFIX: &str = "ModuleInitializer";

fn impl_modules(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name_string = name.to_string();
    let module_name = if name_string.ends_with(TYPE_NAME_SUFFIX) {
        let module_name = name_string.trim_end_matches(TYPE_NAME_SUFFIX);
        if module_name.is_empty() {
            panic!(
                "type name must contain a Lean module name before '{}'",
                TYPE_NAME_SUFFIX
            );
        } else {
            module_name
        }
    } else {
        panic!("type name must end in '{}'", TYPE_NAME_SUFFIX);
    };
    let module_initialization_function_ident =
        format_ident!("initialize_{}", module_name, span = name.span());

    let generated = quote! {
        unsafe impl lean::Modules for #name {
            unsafe fn initialize_modules(builtin: u8, lean_io_world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res {
                unsafe { #module_initialization_function_ident(builtin, lean_io_world) }
            }
        }
    };
    generated.into()
}
