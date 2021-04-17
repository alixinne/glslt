use std::sync::Arc;

use glsl_lang::ast::*;

use indexmap::IndexMap;

use super::instantiate::{CapturedParameter, InstantiateTemplate};
use super::template::TemplateDefinition;
use crate::TransformConfig;

/// Represents a template scope
pub trait Scope: std::fmt::Debug {
    /// Return the current transform configuration
    fn config(&self) -> &TransformConfig;

    /// Return the parent scope, unless this is the global scope
    fn parent_scope(&self) -> Option<&dyn Scope>;

    /// Get the list of defined pointer types in this global scope
    fn declared_pointer_types(&self) -> &IndexMap<SmolStr, FunctionPrototype>;

    /// Get the template corresponding to the given name
    fn get_template(&self, template_name: &str) -> Option<Arc<TemplateDefinition>>;

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
    /// * `definitions`: function definitions introduced by the template instantiation
    fn register_template_instance(&mut self, definitions: Vec<FunctionDefinition>);

    /// Each template instantiation round will generate functions declarations
    /// This allows pulling them in from the parent transformation unit and should be invoked after
    /// each instantiation round.
    fn take_instanced_templates(&mut self) -> Vec<FunctionDefinition>;

    /// Resolve an identifier as a function name. Returns None if this is not possible.
    fn resolve_function_name(&self, name: &str) -> Option<ResolvedArgument>;

    /// Transform a function call to a parameter into an expression
    ///
    /// Returns Err(Error::TransformAsTemplate) if the call can't be transformed by the current
    /// scope and requires a template lookup.
    fn transform_arg_call(
        &mut self,
        expr: &mut Expr,
        instantiator: &mut InstantiateTemplate,
    ) -> crate::Result<()>;

    /// Return the list of parameters captured by this scope
    fn captured_parameters(&self) -> &[CapturedParameter];
}

/// Result of resolving a template parameter
pub struct ResolvedArgument<'fp> {
    pub pointer_type: &'fp FunctionPrototype,
    pub body: ResolvedArgumentExpr,
}

/// Contents of a resolved argument
pub enum ResolvedArgumentExpr {
    /// Static function name
    FunctionName(SmolStr),
    /// Lambda expression
    Lambda(Expr),
}
