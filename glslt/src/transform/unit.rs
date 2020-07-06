use std::collections::HashSet;

use glsl::syntax::*;

use super::instantiate::InstantiateTemplate;
use super::{Context, TransformUnit};

use crate::{Error, Result};

/// GLSLT template instantiation context
pub struct Unit<'c> {
    /// Template definition context
    ctx: &'c mut Context,
    /// Identifiers of already instantiated templates
    instantiated_templates: HashSet<String>,
    /// Result of external declarations copied from input and generated through instantiation
    external_declarations: Vec<ExternalDeclaration>,
    /// Identifiers of function declarations
    known_functions: HashSet<String>,
}

impl<'c> Unit<'c> {
    pub fn new(ctx: &'c mut Context) -> Self {
        let known_functions = ctx.known_functions().clone();

        Self {
            ctx,
            instantiated_templates: HashSet::new(),
            external_declarations: Vec::new(),
            known_functions,
        }
    }

    pub fn into_translation_unit(self) -> Result<TranslationUnit> {
        Ok(TranslationUnit(
            NonEmpty::from_non_empty_iter(self.external_declarations.into_iter())
                .ok_or_else(|| Error::EmptyInput)?,
        ))
    }
}

impl<'c> TransformUnit<'c> for Unit<'c> {
    fn ctx(&self) -> &Context {
        &self.ctx
    }

    fn known_functions(&self) -> &HashSet<String> {
        &self.known_functions
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
        self.known_functions.insert(def.prototype.name.0.clone());

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
