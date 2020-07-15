use std::collections::HashSet;
use std::path::PathBuf;

use glsl::parser::Parse;
use glsl::syntax::*;

use crate::{Error, Result};

fn parse_file(
    path: &PathBuf,
    parsed_external_declarations: &mut Vec<ExternalDeclaration>,
    seen_files: &mut HashSet<PathBuf>,
    include: &[PathBuf],
) -> Result<()> {
    let canonical_path = std::fs::canonicalize(path)?;

    // Get the parent directory of the current file
    let base_path = canonical_path
        .parent()
        .expect("failed to find current directory")
        .to_owned();

    // We've seen this path now
    seen_files.insert(canonical_path.clone());

    // Parse this file
    let tu = TranslationUnit::parse(&std::fs::read_to_string(&canonical_path)?)?;

    // Extend the root TU
    for extdecl in (tu.0).0.into_iter() {
        match extdecl {
            ExternalDeclaration::Preprocessor(pp) => match pp {
                Preprocessor::Include(inc) => {
                    let resolved_path = match &inc.path {
                        Path::Absolute(path) => {
                            let path = PathBuf::from(path);

                            include
                                .iter()
                                .find_map(|dir| std::fs::canonicalize(dir.join(&path)).ok())
                        }
                        Path::Relative(path) => {
                            let path = PathBuf::from(path);

                            std::iter::once(&base_path)
                                .chain(include.iter())
                                .find_map(|dir| std::fs::canonicalize(dir.join(&path)).ok())
                        }
                    };

                    match resolved_path {
                        Some(file) => {
                            if !seen_files.contains(&file) {
                                parse_file(
                                    &file,
                                    parsed_external_declarations,
                                    seen_files,
                                    include,
                                )?;
                            }
                        }
                        None => {
                            return Err(Error::UnresolvedInclude(inc.path));
                        }
                    }
                }
                other => {
                    parsed_external_declarations.push(ExternalDeclaration::Preprocessor(other))
                }
            },
            other => {
                parsed_external_declarations.push(other);
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
/// * `include`: list of include directories for system include resolution
pub fn parse_files(pb: &[PathBuf], include: &[PathBuf]) -> Result<TranslationUnit> {
    let mut parsed_external_declarations = Vec::new();
    let mut seen_files = HashSet::new();

    for path in pb {
        parse_file(
            path,
            &mut parsed_external_declarations,
            &mut seen_files,
            &include,
        )?;
    }

    Ok(TranslationUnit(
        NonEmpty::from_non_empty_iter(parsed_external_declarations.into_iter())
            .ok_or_else(|| Error::EmptyInput)?,
    ))
}
