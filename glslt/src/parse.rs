//! Parsing utilities

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{self, PathBuf};

use glsl_lang::{
    ast::*,
    parse::{Parse, ParseContext, ParseContextData, ParseOptions},
};

/// Filesystem abstraction for include resolving
pub trait PreprocessorFs {
    /// Error type for i/o errors
    type Error: From<glsl_lang::parse::ParseError>;

    /// Read the contents of a file given by its path
    ///
    /// # Parameters
    ///
    /// * `path`: path to the file
    fn read(&self, path: &path::Path) -> Result<Cow<str>, Self::Error>;

    /// Canonicalize the given path
    ///
    /// # Parameters
    ///
    /// * `path`: path to canonicalize
    fn canonicalize(&self, path: &path::Path) -> Result<PathBuf, Self::Error>;

    /// Resolve an include path to an actual file
    ///
    /// # Parameters
    ///
    /// * `base_path`: directory of the current file
    /// * `path`: include path to resolve relative to `base_path`
    fn resolve(&self, base_path: &path::Path, path: &Path) -> Result<PathBuf, Self::Error>;
}

mod std_fs;
pub use std_fs::*;

fn parse_tu_internal<T>(
    base_path: &path::Path,
    tu: TranslationUnit,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
    ctx: &mut ParseContext,
) -> Result<(), T::Error>
where
    T: PreprocessorFs,
{
    // Extend the root TU
    for extdecl in tu.0.into_iter() {
        let Node { content, span } = extdecl;

        match content {
            ExternalDeclarationData::Preprocessor(pp) => match pp.content {
                PreprocessorData::Include(inc) => {
                    let resolved_path = fs.resolve(&base_path, &inc.path)?;
                    if !seen_files.contains(&resolved_path) {
                        parse_file(
                            &resolved_path,
                            parsed_external_declarations,
                            seen_files,
                            fs,
                            ctx,
                        )?;
                    }
                }
                other => parsed_external_declarations.push(ExternalDeclaration::new(
                    ExternalDeclarationData::Preprocessor(Preprocessor::new(other, span)),
                    span,
                )),
            },
            other => {
                parsed_external_declarations.push(Node::new(other, span));
            }
        }
    }

    Ok(())
}

fn parse_file<T>(
    path: &path::Path,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
    ctx: &mut ParseContext,
) -> Result<(), T::Error>
where
    T: PreprocessorFs,
{
    let canonical_path = fs.canonicalize(path)?;

    // Get the parent directory of the current file
    let base_path = canonical_path
        .parent()
        .expect("failed to find current directory")
        .to_owned();

    // We've seen this path now
    seen_files.insert(canonical_path.clone());

    parse_str(
        &base_path,
        &fs.read(&canonical_path)?,
        parsed_external_declarations,
        seen_files,
        fs,
        ctx,
    )
}

fn parse_str<T>(
    base_path: &path::Path,
    source: &str,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
    ctx: &mut ParseContext,
) -> Result<(), T::Error>
where
    T: PreprocessorFs,
{
    // Parse this file
    let (tu, new_ctx) = TranslationUnit::parse_with_options(source, &ctx)?;

    // Swap the options structure with the new one (with new type names and comments)
    *ctx = new_ctx;
    ctx.opts.source_id += 1;

    // Forward the parse process
    parse_tu_internal(
        base_path,
        tu,
        parsed_external_declarations,
        seen_files,
        fs,
        ctx,
    )
}

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

/// Parse the given source with default filesystem inclusion
pub fn parse_source_default(
    source: &str,
) -> Result<(TranslationUnit, ParseContext), StdPreprocessorFsError> {
    let std_fs = StdPreprocessorFs::new();
    let base_path = std::env::current_dir().unwrap();
    parse_source(&base_path, source, &std_fs, None)
}

/// Process the includes of some raw source
pub fn parse_source<T>(
    base_path: &path::Path,
    source: &str,
    fs: &T,
    ctx: Option<&ParseContext>,
) -> Result<(TranslationUnit, ParseContext), T::Error>
where
    T: PreprocessorFs,
{
    let mut parsed_external_declarations = Vec::new();
    let mut seen_files = HashSet::new();

    let mut ctx = make_parse_context(ctx);

    parse_str(
        base_path,
        source,
        &mut parsed_external_declarations,
        &mut seen_files,
        fs,
        &mut ctx,
    )?;

    Ok((
        TranslationUnit(parsed_external_declarations.into_iter().collect()),
        ctx,
    ))
}

/// Parse a set of files into a single translation unit
///
/// # Parameters
///
/// * `pb`: list of paths to concatenate
/// * `fs`: fs implementation
/// * `ctx`: parse options
pub fn parse_files<T>(
    pb: &[PathBuf],
    fs: &T,
    ctx: Option<&ParseContext>,
) -> Result<(TranslationUnit, ParseContext), T::Error>
where
    T: PreprocessorFs,
{
    let mut parsed_external_declarations = Vec::new();
    let mut seen_files = HashSet::new();

    let mut ctx = make_parse_context(ctx);

    for path in pb {
        parse_file(
            path,
            &mut parsed_external_declarations,
            &mut seen_files,
            fs,
            &mut ctx,
        )?;
    }

    Ok((
        TranslationUnit(parsed_external_declarations.into_iter().collect()),
        ctx,
    ))
}
