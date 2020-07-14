use std::collections::HashSet;

use glsl::syntax::*;

use super::Context;

use crate::Result;

pub trait TransformUnit {
    fn ctx(&self) -> &Context;

    fn known_functions(&self) -> &HashSet<String>;

    fn template_instance_declared(&self, template_name: &str) -> bool;

    fn register_template_instance<'s>(&mut self, template_name: &str, instance: FunctionDefinition);

    fn push_function_declaration(&mut self, def: FunctionDefinition);

    fn parse_external_declaration(&mut self, extdecl: ExternalDeclaration) -> Result<()>;
}
