use std::collections::HashMap;
use std::rc::Rc;

use glsl::syntax::*;

use super::template::TemplateDefinition;

/// Represents a template scope
pub trait Scope: std::fmt::Debug {
    /// Return the parent scope, unless this is the global scope
    fn parent_scope(&self) -> Option<&dyn Scope>;

    /// Get the list of defined pointer types in this global scope
    fn declared_pointer_types(&self) -> &HashMap<String, FunctionPrototype>;

    /// Get the template corresponding to the given name
    fn get_template(&self, template_name: &str) -> Option<Rc<TemplateDefinition>>;

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
    fn register_template_instance(
        &mut self,
        template_name: &str,
        instance: Node<FunctionDefinition>,
    );

    /// Each template instantiation round will generate functions declarations
    /// This allows pulling them in from the parent transformation unit and should be invoked after
    /// each instantiation round.
    fn take_instanced_templates(&mut self) -> Vec<Node<FunctionDefinition>>;

    /// Resolve an identifier as a function name. Returns None if this is not possible.
    fn resolve_function_name(&self, name: &str) -> Option<String>;
}
