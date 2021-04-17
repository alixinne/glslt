//! GLSLT transform utilities definitions

use glsl_lang::ast::*;

use crate::Result;

mod fn_ref;
pub use fn_ref::*;

mod global_scope;
pub(crate) use global_scope::*;

mod instantiate;

mod local_scope;
pub(crate) use local_scope::*;

mod min_unit;
pub use min_unit::*;

mod scope;
pub(crate) use scope::*;

pub mod template;

mod transform_unit;
pub use transform_unit::*;

mod unit;
pub use unit::*;

use crate::TransformConfig;

fn transform_unit<'a, T: TransformUnit>(
    asts: impl std::iter::Iterator<Item = &'a TranslationUnit>,
    inst: &mut T,
) -> Result<()> {
    for (_id, ast) in asts.enumerate() {
        // We clone all declarations since they all have somewhere to go
        for extdecl in ast.0.iter().cloned() {
            inst.parse_external_declaration(extdecl)?;
        }
    }

    Ok(())
}

/// Transform a GLSLT AST to an instantiated GLSL AST
///
/// # Parameters
///
/// * `asts`: iterator of translation units to be concatenated and transformed
///
/// # Errors
///
/// Return an error if the transformation fails. See [crate::Error] for possible failure reasons.
pub fn transform<'a>(
    asts: impl std::iter::Iterator<Item = &'a TranslationUnit>,
    config: TransformConfig,
) -> Result<TranslationUnit> {
    let mut inst = Unit::with_config(config);

    transform_unit(asts, &mut inst)?;

    inst.into_translation_unit()
}

/// Transform a GLSLT AST to an instantiated GLSL AST. Only include symbols transitively
/// referenced by any of the entry points listed in `wanted`.
///
/// # Parameters
///
/// * `asts`: iterator of translation units to be concatenated and transformed
/// * `wanted`: list of entry points to keep in the output
///
/// # Errors
///
/// Return an error if the transformation fails. See [crate::Error] for possible failure reasons.
pub fn transform_min<'a>(
    asts: impl std::iter::Iterator<Item = &'a TranslationUnit>,
    wanted: impl std::iter::Iterator<Item = &'a str>,
    config: TransformConfig,
) -> Result<TranslationUnit> {
    let mut inst = MinUnit::with_config(config);

    transform_unit(asts, &mut inst)?;

    inst.into_translation_unit(wanted)
}
