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

/// Create a type that initializes multiple Lean modules
///
/// # Syntax
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
/// ```no_run
/// # extern crate lean;
/// # extern crate lean_sys;
/// #
/// use lean_macro::{
///   combine_lean_module_initializers, create_module_trait
/// };
///
/// enum ParsingTypes {}
///
/// # unsafe impl lean::Modules for ParsingTypes {
/// #     unsafe fn initialize_modules(
/// #         _builtin: u8,
/// #     ) -> lean_sys::lean_obj_res {
/// #         lean::make_lean_io_result_ok_unit()
/// #     }
/// # }
/// #
/// unsafe trait ParsingTypesModule: lean::Modules {}
///
/// # unsafe impl ParsingTypesModule for ParsingTypes {}
/// #
/// enum YamlParser {}
///
/// # unsafe impl lean::Modules for YamlParser {
/// #     unsafe fn initialize_modules(
/// #         _builtin: u8,
/// #     ) -> lean_sys::lean_obj_res {
/// #         lean::make_lean_io_result_ok_unit()
/// #     }
/// # }
/// #
/// unsafe trait YamlParserModule: lean::Modules {}
///
/// # unsafe impl YamlParserModule for YamlParser {}
/// #
/// #[create_module_trait]
/// enum JsonParserModuleInitializer {}
///
/// # unsafe impl lean::Modules for JsonParserModuleInitializer {
/// #     unsafe fn initialize_modules(
/// #         _builtin: u8,
/// #     ) -> lean_sys::lean_obj_res {
/// #         lean::make_lean_io_result_ok_unit()
/// #     }
/// # }
/// #
/// combine_lean_module_initializers! {
///     pub AllParsingModulesInitializer {
///         ParsingTypes : ParsingTypesModule,
///         YamlParser : YamlParserModule,
///         JsonParserModuleInitializer : JsonParserModule,
///     }
/// }
/// ```
///
/// Module initializer types and module initialization traits can have arbitrary
/// names, for flexibility. They do not need to follow the naming conventions
/// imposed by other macros. If some module initializer types do follow the
/// naming conventions, however, then the associated module initialization
/// traits can be omitted. For example:
///
/// ```no_run
/// # extern crate lean;
/// # extern crate lean_sys;
/// #
/// # use lean_macro::{
/// #   combine_lean_module_initializers, create_module_trait
/// # };
/// #
/// # enum ParsingTypes {}
/// #
/// # unsafe impl lean::Modules for ParsingTypes {
/// #     unsafe fn initialize_modules(
/// #         _builtin: u8,
/// #     ) -> lean_sys::lean_obj_res {
/// #         lean::make_lean_io_result_ok_unit()
/// #     }
/// # }
/// #
/// # unsafe trait ParsingTypesModule: lean::Modules {}
/// #
/// # unsafe impl ParsingTypesModule for ParsingTypes {}
/// #
/// # enum YamlParser {}
/// #
/// # unsafe impl lean::Modules for YamlParser {
/// #     unsafe fn initialize_modules(
/// #         _builtin: u8,
/// #     ) -> lean_sys::lean_obj_res {
/// #         lean::make_lean_io_result_ok_unit()
/// #     }
/// # }
/// #
/// # unsafe trait YamlParserModule: lean::Modules {}
/// #
/// # unsafe impl YamlParserModule for YamlParser {}
/// #
/// # #[create_module_trait]
/// # enum JsonParserModuleInitializer {}
/// #
/// # unsafe impl lean::Modules for JsonParserModuleInitializer {
/// #     unsafe fn initialize_modules(
/// #         _builtin: u8,
/// #     ) -> lean_sys::lean_obj_res {
/// #         lean::make_lean_io_result_ok_unit()
/// #     }
/// # }
/// #
/// combine_lean_module_initializers! {
///     pub AllParsingModulesInitializer {
///         ParsingTypes : ParsingTypesModule,
///         YamlParser : YamlParserModule,
///         JsonParserModuleInitializer, // Simplified because the type has a known suffix
///     }
/// }
/// ```
#[proc_macro]
pub fn combine_lean_module_initializers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = parse_macro_input!(input as CombineLeanModuleInitializers);
    let expanded = combine_lean_module_initializers::generate(parsed_input);

    proc_macro::TokenStream::from(expanded)
}
