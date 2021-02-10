//! Parsing utilities

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

use glsl_lang::{
    ast::*,
    parse::{Parse, ParseOptions},
};

/// Filesystem abstraction for include resolving
pub trait PreprocessorFs {
    /// Error type for i/o errors
    type Error: From<glsl_lang::parse::ParseErrorStatic>;

    /// Read the contents of a file given by its path
    ///
    /// # Parameters
    ///
    /// * `path`: path to the file
    fn read(&self, path: &PathBuf) -> Result<Cow<str>, Self::Error>;

    /// Canonicalize the given path
    ///
    /// # Parameters
    ///
    /// * `path`: path to canonicalize
    fn canonicalize(&self, path: &PathBuf) -> Result<PathBuf, Self::Error>;

    /// Resolve an include path to an actual file
    ///
    /// # Parameters
    ///
    /// * `base_path`: directory of the current file
    /// * `path`: include path to resolve relative to `base_path`
    fn resolve(&self, base_path: &PathBuf, path: &Path) -> Result<PathBuf, Self::Error>;
}

mod std_fs;
pub use std_fs::*;

fn parse_tu_internal<T>(
    base_path: &PathBuf,
    tu: TranslationUnit,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
    opts: &mut ParseOptions,
) -> Result<(), T::Error>
where
    T: PreprocessorFs,
{
    // Extend the root TU
    for extdecl in tu.0.into_iter() {
        let Node { contents, span } = extdecl;

        match contents {
            ExternalDeclarationData::Preprocessor(pp) => match pp.contents {
                PreprocessorData::Include(inc) => {
                    let resolved_path = fs.resolve(&base_path, &inc.path)?;
                    if !seen_files.contains(&resolved_path) {
                        parse_file(
                            &resolved_path,
                            parsed_external_declarations,
                            seen_files,
                            fs,
                            opts,
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
    path: &PathBuf,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
    opts: &mut ParseOptions,
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
        opts,
    )
}

fn parse_str<T>(
    base_path: &PathBuf,
    source: &str,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
    opts: &mut ParseOptions,
) -> Result<(), T::Error>
where
    T: PreprocessorFs,
{
    // Parse this file
    let (tu, new_opts) =
        TranslationUnit::parse_with_options(source, &opts).map_err(|e| e.map_token(Into::into))?;

    // Swap the options structure with the new one (with new type names and comments)
    *opts = new_opts;
    opts.source_id += 1;

    // Forward the parse process
    parse_tu_internal(
        base_path,
        tu,
        parsed_external_declarations,
        seen_files,
        fs,
        opts,
    )
}

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

fn get_parse_options() -> ParseOptions {
    ParseOptions {
        type_names: glsl_lang::parse::TypeNames::with_policy(GlsltPolicy),
        comments: Some(Default::default()),
        ..Default::default()
    }
}

/// Parse the given source with default filesystem inclusion
pub fn parse_source_default(
    source: &str,
) -> Result<(TranslationUnit, ParseOptions), StdPreprocessorFsError> {
    let std_fs = StdPreprocessorFs::new();
    let base_path = std::env::current_dir().unwrap();
    Ok(parse_source(&base_path, source, &std_fs)?)
}

/// Process the includes of some raw source
pub fn parse_source<T>(
    base_path: &PathBuf,
    source: &str,
    fs: &T,
) -> Result<(TranslationUnit, ParseOptions), T::Error>
where
    T: PreprocessorFs,
{
    let mut parsed_external_declarations = Vec::new();
    let mut seen_files = HashSet::new();

    let mut opts = get_parse_options();

    parse_str(
        base_path,
        source,
        &mut parsed_external_declarations,
        &mut seen_files,
        fs,
        &mut opts,
    )?;

    Ok((
        TranslationUnit(parsed_external_declarations.into_iter().collect()),
        opts,
    ))
}

/// Parse a set of files into a single translation unit
///
/// # Parameters
///
/// * `pb`: list of paths to concatenate
/// * `fs`: fs implementation
pub fn parse_files<T>(pb: &[PathBuf], fs: &T) -> Result<(TranslationUnit, ParseOptions), T::Error>
where
    T: PreprocessorFs,
{
    let mut parsed_external_declarations = Vec::new();
    let mut seen_files = HashSet::new();

    let mut opts = get_parse_options();

    for path in pb {
        parse_file(
            path,
            &mut parsed_external_declarations,
            &mut seen_files,
            fs,
            &mut opts,
        )?;
    }

    Ok((
        TranslationUnit(parsed_external_declarations.into_iter().collect()),
        opts,
    ))
}
