use std::cmp::Ord;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    Ident, Token, Visibility, braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
};

/// Parses the following syntax:
///
/// ```text
/// combine_lean_module_initializers! {
///     $VISIBILITY $STRUCT_NAME {
///         $MODULE_INITIALIZER_TYPE1 : $MODULE_TRAIT1,
///         $MODULE_INITIALIZER_TYPE2 : $MODULE_TRAIT2,
///         ...
///         $MODULE_INITIALIZER_TYPEN : $MODULE_TRAITN(,)
///     }
/// }
/// ```
///
/// Where `MODULE_INITIALIZER_TYPE` identifiers are types that implement
/// `lean::Modules` and also implement the associated `MODULE_TRAIT` traits,
/// which have `lean::Modules` as a supertrait.
///
/// For example:
///
/// ```text
/// combine_lean_module_initializers! {
///     pub AllParsingModulesInitializer {
///         ParsingTypes : ParsingTypesModule,
///         YamlParser : YamlParserModule,
///         JsonParserModuleInitializer : JsonParser,
///     }
/// }
/// ```
///
/// Module initializer types and module initialization traits can have arbitrary
/// names, for flexibility. They do not need to follow the naming conventions
/// imposed by other macros.
pub struct CombineLeanModuleInitializers {
    visibility: Visibility,
    name: Ident,
    module_initializers: Vec<ModuleInitializer>,
    module_traits: Vec<ModuleTrait>,
}

impl Parse for CombineLeanModuleInitializers {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility: Visibility = input.parse()?;
        let name: Ident = input.parse()?;

        let fields;
        braced!(fields in input);
        let modules_punctuated: Punctuated<ModuleInitializationPair, Token![,]> =
            fields.parse_terminated(ModuleInitializationPair::parse, Token![,])?;

        let (mut module_initializers, mut module_traits): (Vec<_>, Vec<_>) = modules_punctuated
            .into_iter()
            .map(ModuleInitializationPair::into_tuple)
            .unzip();

        if module_initializers.is_empty() {
            return Err(syn::Error::new(
                name.span(),
                "at least one module initializer type and trait is required, otherwise use `lean::NoModules`",
            ));
        }

        module_initializers.sort();
        if let Some(duplicate) = find_first_duplicate_in_sorted_slice(&module_initializers) {
            return Err(syn::Error::new(
                duplicate.0.span(),
                format!("duplicate module initializer type `{}`", duplicate.0),
            ));
        }

        module_traits.sort();
        if let Some(duplicate) = find_first_duplicate_in_sorted_slice(&module_traits) {
            return Err(syn::Error::new(
                duplicate.0.span(),
                format!("duplicate module trait `{}`", duplicate.0),
            ));
        }

        Ok(Self {
            visibility,
            name,
            module_initializers,
            module_traits,
        })
    }
}

fn find_first_duplicate_in_sorted_slice<T: Eq>(items: &[T]) -> Option<&T> {
    for (a, b) in items.iter().zip(items.iter().skip(1)) {
        if a == b {
            return Some(b);
        }
    }
    None
}

struct ModuleInitializationPair {
    module_initializer: Ident,
    module_trait: Ident,
}

impl Parse for ModuleInitializationPair {
    fn parse(input: ParseStream) -> Result<Self> {
        let module_initializer: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let module_trait: Ident = input.parse()?;

        Ok(Self {
            module_initializer,
            module_trait,
        })
    }
}

impl ModuleInitializationPair {
    fn into_tuple(self) -> (ModuleInitializer, ModuleTrait) {
        (
            ModuleInitializer(self.module_initializer),
            ModuleTrait(self.module_trait),
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ModuleInitializer(Ident);

impl ToTokens for ModuleInitializer {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let module_initializer = &self.0;
        let generated = quote! {
          result = #module_initializer::initialize_modules(builtin, lean_io_world);
          if ::lean_sys::lean_io_result_is_ok(result) {
              ::lean_sys::lean_dec(result);
          } else {
              return result;
          }
        };
        tokens.append_all(generated);
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ModuleTrait(Ident);

impl ModuleTrait {
    fn add_tokens(&self, container_name: &Ident, tokens: &mut TokenStream2) {
        let trait_name = &self.0;
        let generated = quote! {
          unsafe impl #trait_name for #container_name {}
        };
        tokens.append_all(generated);
    }
}

pub fn generate(input: CombineLeanModuleInitializers) -> TokenStream2 {
    let CombineLeanModuleInitializers {
        visibility,
        name,
        module_initializers,
        module_traits,
    } = input;

    let mut generated = quote! {
        #visibility enum #name {}

        unsafe impl ::lean::Modules for #name {
            unsafe fn initialize_modules(builtin: u8, lean_io_world: ::lean_sys::lean_obj_arg) -> ::lean_sys::lean_obj_res {
              let mut result: *mut ::lean_sys::lean_object;
              #(#module_initializers)*
              result
            }
        }
    };

    for module_trait in module_traits {
        module_trait.add_tokens(&name, &mut generated);
    }

    generated
}
