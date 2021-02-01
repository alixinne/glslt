use glsl::syntax::*;
use thiserror::Error;

/// GLSLT transform error type
#[derive(Debug, Error)]
pub enum Error {
    /// Re-declaration of a function pointer type
    #[error("function pointer {name} previously declared as {previous_declaration}")]
    DuplicatePointerDefinition {
        /// Name of the duplicated pointer definition
        name: String,
        /// Source code of the previous declaration
        previous_declaration: String,
    },
    /// Template parameter declared with an array specifier
    #[error("template {name} parameter #{index} cannot have an array specifier")]
    ArrayedTemplateParameter {
        /// Name of the template parameter
        name: String,
        /// 0-based position of the parameter in the function declaration
        index: usize,
    },
    /// Unsupported preprocessor directive in minifying mode
    #[error("unsupported preprocessor directive in minifying mode: {0:?}")]
    UnsupportedPreprocessor(Preprocessor),
    /// Unsupported declarator list in minifying mode
    #[error("unsupported declarator list in minifying mode: {0:?}")]
    UnsupportedIDL(InitDeclaratorList),
    /// Undeclared pointer type
    #[error("undeclared pointer type: {0}")]
    UndeclaredPointerType(String),
    /// Could not transform the call from available arguments
    #[error("try to transform this call as a template call")]
    TransformAsTemplate,
    /// Invalid parameter in template argument call
    #[error("invalid argument at index {0}")]
    InvalidParameter(usize),
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
    /// Create a new [DuplicatePointerDefinition](#variant.DuplicatePointerDefinition)
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
