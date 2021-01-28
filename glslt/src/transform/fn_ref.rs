//! FnRef type declaration

use std::rc::Rc;

use glsl::syntax::*;

use crate::transform::template::TemplateDefinition;

/// Reference to a function held in a transformation unit
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FnRef<'a> {
    /// Prototype of the target function
    pub prototype: &'a FunctionPrototype,
    /// Contents of the target function
    pub statement: &'a CompoundStatement,
}

impl glsl::syntax::NodeContents for FnRef<'_> {}

/// Handle to a function definition somewhere in the glslt units
#[derive(Debug, Clone)]
pub enum FnHandle<'a> {
    /// Reference to a template definition
    Owned(Rc<TemplateDefinition>),
    /// Reference to a function declaration
    Borrowed(Node<FnRef<'a>>),
}

impl From<Rc<TemplateDefinition>> for FnHandle<'_> {
    fn from(td: Rc<TemplateDefinition>) -> Self {
        Self::Owned(td)
    }
}

impl<'a> From<Node<FnRef<'a>>> for FnHandle<'a> {
    fn from(r: Node<FnRef<'a>>) -> Self {
        Self::Borrowed(r)
    }
}

impl<'a> FnHandle<'a> {
    /// Create a FnRef for this handle
    pub fn fn_ref(&self) -> Node<FnRef> {
        match self {
            Self::Owned(r) => r.fn_ref(),
            Self::Borrowed(r) => r.clone(),
        }
    }
}
