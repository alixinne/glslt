//! Parsing utilities

use std::path::{Path, PathBuf};

use glsl_lang::{
    ast,
    parse::{ParseContext, ParseContextData, ParseOptions},
};
use glsl_lang_pp::processor::event::Located;

#[derive(Debug, Clone, Copy, PartialEq)]
struct GlsltPolicy;

impl glsl_lang::parse::TypeTablePolicy for GlsltPolicy {
    fn promote_to_type_name(
        &self,
        name: &glsl_lang::ast::Identifier,
        ctx: glsl_lang::parse::IdentifierContext,
    ) -> bool {
        trace!("glslt parsing: promoting {} to type name", name);
        ctx == glsl_lang::parse::IdentifierContext::FunctionPrototype
    }
}

fn make_parse_context(existing: Option<&ParseContext>) -> ParseContext {
    if let Some(existing) = existing {
        existing.with_policy(GlsltPolicy)
    } else {
        ParseOptions::new().with_context(ParseContextData::with_comments_and_policy(GlsltPolicy))
    }
}

/// Parse the given source with the default options and no filesystem inclusion
pub fn parse_source_default(
    source: &str,
) -> glsl_lang::parse::ParseResult<&str, ast::TranslationUnit> {
    parse_source(source, None)
}

/// Parse the given source without filesystem inclusion
pub fn parse_source<'i>(
    source: &'i str,
    ctx: Option<&ParseContext>,
) -> glsl_lang::parse::ParseResult<&'i str, ast::TranslationUnit> {
    use glsl_lang::parse::IntoLexerExt;

    let parse_context = make_parse_context(ctx);
    source.builder().opts(&parse_context).parse()
}

/// Parse a set of files into a single translation unit
///
/// # Parameters
///
/// * `pb`: list of paths to concatenate
/// * `fs`: fs implementation
/// * `ctx`: parse options
pub fn parse_files<'p, F: glsl_lang_pp::processor::fs::FileSystem>(
    pb: &[PathBuf],
    system_paths: &[impl AsRef<Path>],
    fs: F,
    ctx: Option<&ParseContext>,
) -> Result<
    (ast::TranslationUnit, ParseContext),
    lang_util::error::ParseError<glsl_lang::lexer::v2::LexicalError<F::Error>>,
> {
    use glsl_lang::{lexer::v2::fs::PreprocessorExt, parse::IntoLexerExt};

    let mut processor = glsl_lang_pp::processor::fs::Processor::new_with_fs(fs);
    processor
        .system_paths_mut()
        .extend(system_paths.iter().map(|path| path.as_ref().to_owned()));

    let mut external_declarations = Vec::new();

    let mut ctx = make_parse_context(ctx);

    for path in pb {
        let (tu, new_ctx, _) = processor
            .open(path, None)
            .map_err(|err| {
                glsl_lang::lexer::v2::LexicalError::Io(Located::new_at_file(err, path.to_owned()))
            })?
            .builder::<'_, '_, ast::TranslationUnit>()
            .opts(&ctx)
            .parse()?;

        ctx = new_ctx;
        external_declarations.extend(tu.0);
    }

    Ok((ast::TranslationUnit(external_declarations), ctx))
}
