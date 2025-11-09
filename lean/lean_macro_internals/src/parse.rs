use std::error::Error;
use std::fmt;

use proc_macro2::Ident;
use quote::format_ident;

const TRAIT_NAME_SUFFIX: &str = "Module";
const TYPE_NAME_SUFFIX: &str = "ModuleInitializer";

#[derive(Debug, Eq, PartialEq)]
enum ModulesTypeNameError {
    MissingModuleName,
    MissingSuffix,
}

impl fmt::Display for ModulesTypeNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("type name must ")?;
        match self {
            Self::MissingSuffix => f.write_str("end in"),
            Self::MissingModuleName => f.write_str("contain a Lean module name before"),
        }?;
        write!(f, " '{}'", TYPE_NAME_SUFFIX)
    }
}

impl Error for ModulesTypeNameError {}

fn parse_lean_module_name_from_rust_module_initializer_type_name_inner(
    name: &Ident,
) -> Result<String, ModulesTypeNameError> {
    let name_string = name.to_string();
    if name_string.ends_with(TYPE_NAME_SUFFIX) {
        let module_name = name_string.trim_end_matches(TYPE_NAME_SUFFIX);
        if module_name.is_empty() {
            Err(ModulesTypeNameError::MissingModuleName)
        } else {
            Ok(module_name.to_string())
        }
    } else {
        Err(ModulesTypeNameError::MissingSuffix)
    }
}

fn parse_lean_module_name_from_rust_module_initializer_type_name(
    name: &Ident,
) -> syn::Result<String> {
    parse_lean_module_name_from_rust_module_initializer_type_name_inner(name)
        .map_err(|error| syn::Error::new(name.span(), error))
}

pub fn parse_lean_module_initialization_function_from_rust_module_initializer_type_name(
    name: &Ident,
) -> syn::Result<Ident> {
    let module_name = parse_lean_module_name_from_rust_module_initializer_type_name(name)?;
    Ok(format_ident!(
        "initialize_{}",
        module_name,
        span = name.span()
    ))
}

pub fn parse_lean_module_trait_from_rust_module_initializer_type_name(
    name: &Ident,
) -> syn::Result<Ident> {
    let module_name = parse_lean_module_name_from_rust_module_initializer_type_name(name)?;
    let trait_name = format_ident!("{}{}", module_name, TRAIT_NAME_SUFFIX, span = name.span());
    Ok(trait_name)
}
