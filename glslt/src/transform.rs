use glsl::syntax::*;

use crate::Result;

mod context;
use context::Context;

mod instantiate;

mod template;

pub fn transform<'a>(
    asts: impl std::iter::Iterator<Item = &'a TranslationUnit>,
) -> Result<TranslationUnit> {
    let mut ctx = Context::default();

    for (_id, ast) in asts.enumerate() {
        // We clone all declarations since they all have somewhere to go
        for extdecl in (ast.0).0.iter().cloned() {
            ctx.parse_external_declaration(extdecl)?;
        }
    }

    ctx.into_translation_unit()
}
