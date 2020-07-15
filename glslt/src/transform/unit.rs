use std::collections::HashSet;

use glsl::syntax::*;

use super::instantiate::InstantiateTemplate;
use super::{Context, TransformUnit};

use crate::{Error, Result};

/// GLSLT template instantiation context
#[derive(Default, Debug, Clone)]
pub struct Unit {
    /// Template definition context
    ctx: Context,
    /// Identifiers of already instantiated templates
    instantiated_templates: HashSet<String>,
    /// Result of external declarations copied from input and generated through instantiation
    external_declarations: Vec<ExternalDeclaration>,
}

impl Unit {
    /// Create a new transform unit
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
            instantiated_templates: HashSet::new(),
            external_declarations: Vec::new(),
        }
    }

    /// Create a new transform unit using pre-defined templates
    ///
    /// # Parameters
    ///
    /// * `ctx`: context to pull pre-defined templates from
    pub fn with_context(ctx: Context) -> Self {
        Self {
            ctx,
            instantiated_templates: HashSet::new(),
            external_declarations: Vec::new(),
        }
    }

    /// Transform this unit into a GLSL syntax tree
    ///
    /// All the declarations given as input will be included in-order in the output, with the
    /// template instantiations interleaved when needed.
    pub fn into_translation_unit(self) -> Result<TranslationUnit> {
        Ok(TranslationUnit(
            NonEmpty::from_non_empty_iter(self.external_declarations.into_iter())
                .ok_or_else(|| Error::EmptyInput)?,
        ))
    }
}

impl TransformUnit for Unit {
    fn ctx(&self) -> &Context {
        &self.ctx
    }

    fn known_functions(&self) -> &HashSet<String> {
        self.ctx.known_functions()
    }

    fn template_instance_declared(&self, template_name: &str) -> bool {
        self.instantiated_templates.contains(template_name)
    }

    fn register_template_instance<'s>(
        &mut self,
        template_name: &str,
        instance: FunctionDefinition,
    ) {
        // Instantiate the template and add it to the declarations before us
        self.external_declarations
            .push(ExternalDeclaration::FunctionDefinition(instance));

        // Take note we instantiated the template
        self.instantiated_templates.insert(template_name.to_owned());
    }

    fn push_function_declaration(&mut self, def: FunctionDefinition) {
        self.ctx
            .known_functions_mut()
            .insert(def.prototype.name.0.clone());

        // Add the definition to the declarations
        self.external_declarations
            .push(ExternalDeclaration::FunctionDefinition(def));
    }

    fn parse_external_declaration(&mut self, extdecl: ExternalDeclaration) -> Result<()> {
        if let Some(extdecl) = self.ctx.parse_external_declaration(extdecl)? {
            match extdecl {
                ExternalDeclaration::FunctionDefinition(def) => {
                    // No template parameter, it's a "regular" function so it has to be
                    // processed to instantiate parameters
                    //
                    // TODO: Recursive template instantiation?
                    InstantiateTemplate::new(self).instantiate(def)?;
                }
                other => self.external_declarations.push(other),
            }
        }

        Ok(())
    }
}
