use glsl::syntax::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no top-level declarations were found in any of the input files")]
    EmptyInput,
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("parse error: {0}")]
    ParseError(#[from] glsl::parser::ParseError),
    #[error("function pointer {name} previously declared as {previous_declaration}")]
    DuplicatePointerDefinition {
        name: String,
        previous_declaration: String,
    },
    #[error("template {name} parameter #{index} cannot have an array specifier")]
    ArrayedTemplateParameter { name: String, index: usize },
    #[error("invalid template parameter #{index} for {name} instantiation")]
    InvalidTemplateParameter { name: String, index: usize },
    #[error("unresolved include: {0:?}")]
    UnresolvedInclude(glsl::syntax::Path),
}

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
