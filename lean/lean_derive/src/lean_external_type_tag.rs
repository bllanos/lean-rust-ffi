use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

pub fn impl_lean_external_type_tag(input: TokenStream2) -> syn::Result<TokenStream2> {
    let derive_input: DeriveInput = syn::parse2(input)?;
    let name = &derive_input.ident;

    let generated = quote! {
        unsafe impl ::lean::lean_types::external::LeanExternalTypeTag for #name {
            type InternalLeanObjectIterator = ::std::iter::Empty<::lean_sys::b_lean_obj_arg>;

            fn iter(&self) -> Self::InternalLeanObjectIterator {
                ::std::iter::empty()
            }
        }
    };
    Ok(generated)
}
