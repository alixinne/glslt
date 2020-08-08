//! FnRef type declaration

use glsl::syntax::*;

/// Reference to a function held in a transformation unit
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FnRef<'a> {
    /// Prototype of the target function
    pub prototype: &'a FunctionPrototype,
    /// Contents of the target function
    pub statement: &'a CompoundStatement,
}

impl glsl::syntax::NodeContents for FnRef<'_> {}
