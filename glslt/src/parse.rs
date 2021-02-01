use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

use glsl::parser::Parse;
use glsl::syntax::*;

/// Filesystem abstraction for include resolving
pub trait PreprocessorFs {
    /// Error type for i/o errors. Needs to be convertible from glsl::parser::ParseError
    type Error;

    /// Read the contents of a file given by its path
    ///
    /// # Parameters
    ///
    /// * `path`: path to the file
    fn read(&self, path: &std::path::Path) -> Result<Cow<str>, Self::Error>;

    /// Canonicalize the given path
    ///
    /// # Parameters
    ///
    /// * `path`: path to canonicalize
    fn canonicalize(&self, path: &std::path::Path) -> Result<PathBuf, Self::Error>;

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

fn parse_file<T>(
    path: &PathBuf,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    fs: &T,
) -> Result<(), T::Error>
where
    T: PreprocessorFs,
    T::Error: From<glsl::parser::ParseError>
{
    let canonical_path = fs.canonicalize(path)?;

    // Get the parent directory of the current file
    let base_path = canonical_path
        .parent()
        .expect("failed to find current directory")
        .to_owned();

    // We've seen this path now
    seen_files.insert(canonical_path.clone());

    // Parse this file
    let tu = TranslationUnit::parse(&fs.read(&canonical_path)?)?;

    // Extend the root TU
    for extdecl in (tu.0).0.into_iter() {
        let Node { contents, span } = extdecl;

        match contents {
            ExternalDeclarationData::Preprocessor(pp) => match pp.contents {
                PreprocessorData::Include(inc) => {
                    let resolved_path = fs.resolve(&base_path, &inc.path)?;
                    if !seen_files.contains(&resolved_path) {
                        parse_file(&resolved_path, parsed_external_declarations, seen_files, fs)?;
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

/// Parse a set of files into a single translation unit
///
/// # Parameters
///
/// * `pb`: list of paths to concatenate
/// * `fs`: fs implementation
pub fn parse_files<T>(pb: &[PathBuf], fs: &T) -> Result<TranslationUnit, T::Error>
where
    T: PreprocessorFs,
    T::Error: From<glsl::parser::ParseError>
{
    let mut parsed_external_declarations = Vec::new();
    let mut seen_files = HashSet::new();

    for path in pb {
        parse_file(path, &mut parsed_external_declarations, &mut seen_files, fs)?;
    }

    Ok(TranslationUnit(NonEmpty(parsed_external_declarations.into_iter().collect())))
}
