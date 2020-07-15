use std::collections::HashSet;

use glsl::syntax::*;

use super::Context;

use crate::Result;

/// Represent a GLSLT transform unit
///
/// The entry point of transform units is the
/// [parse_external_declaration](#method.parse_external_declaration), which processes its input
/// syntax as GLSLT and updates the internal state of the transform unit accordingly. The other
/// methods are for internal operation.
pub trait TransformUnit {
    /// Obtain a reference to the template definition context
    fn ctx(&self) -> &Context;

    /// Obtain a reference to the known function names list
    fn known_functions(&self) -> &HashSet<String>;

    /// Determine if a template has been instantiated with the given name
    ///
    /// # Parameters
    ///
    /// * `template_name`: name of the template instantiation
    fn template_instance_declared(&self, template_name: &str) -> bool;

    /// Register a template instantiation in the current transform unit
    ///
    /// # Parameters
    ///
    /// * `template_name`: name of the template instantiation
    /// * `instance`: function definition corresponding to the instantiation
    fn register_template_instance(&mut self, template_name: &str, instance: FunctionDefinition);

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
    fn parse_external_declaration(&mut self, extdecl: ExternalDeclaration) -> Result<()>;
}
