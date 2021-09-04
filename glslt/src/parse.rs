//! Parsing utilities

use glsl_lang::lexer::{ParseContext, ParseContextData};

#[derive(Debug, Clone, Copy, PartialEq)]
struct GlsltPolicy;

impl glsl_lang::lexer::TypeTablePolicy for GlsltPolicy {
    fn promote_to_type_name(
        &self,
        name: &glsl_lang::ast::Identifier,
        ctx: glsl_lang::lexer::IdentifierContext,
    ) -> bool {
        trace!("glslt parsing: promoting {} to type name", name);
        ctx == glsl_lang::lexer::IdentifierContext::FunctionPrototype
    }
}

/// Create a parsing context that follows GLSLT's type rules
///
/// # Parameters
///
/// * `existing`: optional existing parsing context to reuse
pub fn make_parse_context(existing: Option<&ParseContext>) -> ParseContext {
    if let Some(existing) = existing {
        existing.with_policy(GlsltPolicy)
    } else {
        ParseContext::new_with_context(ParseContextData::with_comments_and_policy(GlsltPolicy))
    }
}
