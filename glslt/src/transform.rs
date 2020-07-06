//! GLSLT transform utilities definitions

use glsl::syntax::*;

use crate::Result;

mod context;
pub use context::*;

mod instantiate;

pub mod template;

mod unit;
pub use unit::*;

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
) -> Result<TranslationUnit> {
    let mut ctx = Context::default();
    let mut inst = Unit::new(&mut ctx);

    for (_id, ast) in asts.enumerate() {
        // We clone all declarations since they all have somewhere to go
        for extdecl in (ast.0).0.iter().cloned() {
            inst.parse_external_declaration(extdecl)?;
        }
    }

    inst.into_translation_unit()
}
