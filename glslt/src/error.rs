use glsl::syntax::*;
use thiserror::Error;

/// GLSLT transform error type
#[derive(Debug, Error)]
pub enum Error {
    /// No top-level declarations were found in any of the input files
    #[error("no top-level declarations were found in any of the input files")]
    EmptyInput,
    /// I/O error
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),
    /// GLSL parse error
    #[error("parse error: {0}")]
    ParseError(#[from] glsl::parser::ParseError),
    /// Re-declaration of a function pointer type
    #[error("function pointer {name} previously declared as {previous_declaration}")]
    DuplicatePointerDefinition {
        name: String,
        previous_declaration: String,
    },
    /// Template parameter declared with an array specifier
    #[error("template {name} parameter #{index} cannot have an array specifier")]
    ArrayedTemplateParameter { name: String, index: usize },
    /// Unresolved include directive
    #[error("unresolved include: {0:?}")]
    UnresolvedInclude(glsl::syntax::Path),
}

/// GLSLT Result
pub type Result<T> = std::result::Result<T, Error>;

fn prototype_to_string(prototype: &FunctionPrototype) -> String {
    let mut s = String::new();
    glsl::transpiler::glsl::show_function_prototype(
        &mut s,
        prototype,
        &mut glsl::transpiler::glsl::FormattingState::default(),
    )
    .unwrap();
    s
}

impl Error {
    /// Create a new [Error::DuplicatePointerDefinition]
    ///
    /// # Parameters
    ///
    /// * `name`: name of the duplicated identifier
    /// * `previous`: previous declaration of the pointer
    pub fn new_duplicate_pointer_definition(
        name: &Identifier,
        previous: &FunctionPrototype,
    ) -> Self {
        Error::DuplicatePointerDefinition {
            name: name.0.clone(),
            previous_declaration: prototype_to_string(&previous),
        }
    }
}
