use glsl_lang::ast::*;

use super::{FnHandle, GlobalScope};

use crate::Result;

/// Represent a GLSLT transform unit
///
/// The entry point of transform units is the
/// [parse_external_declaration](#method.parse_external_declaration), which processes its input
/// syntax as GLSLT and updates the internal state of the transform unit accordingly. The other
/// methods are for internal operation.
pub trait TransformUnit {
    /// Obtain a reference to the template definition context
    fn global_scope(&self) -> &GlobalScope;

    /// Obtain a mutable reference to the template definition context
    fn global_scope_mut(&mut self) -> &mut GlobalScope;

    /// Add a function declaration to the transform unit
    ///
    /// # Parameters
    ///
    /// * `def`: syntax tree for the function definition
    fn push_function_declaration(&mut self, def: FunctionDefinition);

    /// Parse a GLSLT declaration and add it to the transform unit
    ///
    /// # Parameters
    ///
    /// * `extdecl`: top-level declaration to parse as GLSLT
    fn parse_external_declaration(
        &mut self,
        extdecl: ExternalDeclaration,
    ) -> Result<Option<FnHandle>>;
}
