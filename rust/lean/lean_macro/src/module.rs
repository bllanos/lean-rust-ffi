use proc_macro2::TokenStream as TokenStream2;
use quote::{TokenStreamExt, format_ident, quote};
use syn::{DeriveInput, spanned::Spanned};

use lean_macro_internals::parse;

const ATTRIBUTE_DESCRIPTION: &str = "`create_module_trait` attribute";

pub fn impl_create_module_trait(
    input: TokenStream2,
    annotated_item: TokenStream2,
) -> syn::Result<TokenStream2> {
    if !input.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            format!("{} does not take any input", ATTRIBUTE_DESCRIPTION),
        ));
    }

    let mut generated = annotated_item.clone();
    let derive_input: DeriveInput = syn::parse2(annotated_item)?;
    let name = &derive_input.ident;
    let module_name = parse::parse_lean_module_name_from_rust_module_initializer_type_name(name)
        .map_err(|mut error| {
            error.combine(syn::Error::new(
                name.span(),
                format!("error using {}", ATTRIBUTE_DESCRIPTION),
            ));
            error
        })?;
    let trait_name = format_ident!("{}Module", module_name, span = name.span());

    let appended = quote! {
        /// A trait implemented by types that initialize this Lean module
        ///
        /// # Safety
        ///
        /// Implementations of this trait must guarantee that the module is properly
        /// initialized.
        // This should not be necessary. Perhaps there is a bug in Clippy?
        #[allow(clippy::missing_safety_doc)]
        pub unsafe trait #trait_name: lean::Modules {}

        unsafe impl #trait_name for #name {}
    };

    generated.append_all(appended);
    Ok(generated)
}
