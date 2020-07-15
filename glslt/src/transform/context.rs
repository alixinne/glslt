use std::collections::{HashMap, HashSet};

use glsl::syntax::*;

use super::template::{TemplateDefinition, TryTemplate};
use crate::{Error, Result};

/// GLSLT template definition context
#[derive(Default, Debug, Clone)]
pub struct Context {
    /// Known pointer types
    declared_pointer_types: HashMap<String, FunctionPrototype>,
    /// Known GLSLT template functions
    declared_templates: HashMap<String, TemplateDefinition>,
    /// Identifiers of function declarations
    known_functions: HashSet<String>,
}

impl Context {
    /// Create a new template definition context
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_function_prototype(&mut self, prototype: FunctionPrototype) -> Result<()> {
        if let Some(previous) = self.declared_pointer_types.get(&prototype.name.0) {
            // Since function pointer types are raw identifiers, they have to be unique
            return Err(Error::new_duplicate_pointer_definition(
                &prototype.name,
                previous,
            ));
        } else {
            info!("declared pointer: {}", prototype.name.0);

            self.declared_pointer_types
                .insert(prototype.name.0.clone(), prototype);
        }

        Ok(())
    }

    fn parse_declaration(&mut self, decl: Declaration) -> Result<Option<ExternalDeclaration>> {
        match decl {
            Declaration::FunctionPrototype(prototype) => {
                // A function prototype is what we'll call a function pointer type
                self.parse_function_prototype(prototype)?;
                Ok(None)
            }
            other => Ok(Some(ExternalDeclaration::Declaration(other))),
        }
    }

    fn parse_function_definition(
        &mut self,
        def: FunctionDefinition,
    ) -> Result<Option<FunctionDefinition>> {
        // A function definition is a template if any of its arguments is a pointer
        let name = def.prototype.name.0.clone();
        let template =
            super::template::parse_definition_as_template(def, &self.declared_pointer_types)?;

        match template {
            TryTemplate::Template(template) => {
                info!("declared template: {}", template.ast.prototype.name.0);

                // We found a template parameter, so it's a template function
                self.declared_templates.insert(name, template);
                Ok(None)
            }
            TryTemplate::Function(def) => Ok(Some(def)),
        }
    }

    /// Get the list of defined function identifiers in this context
    pub fn known_functions(&self) -> &HashSet<String> {
        &self.known_functions
    }

    /// Get the list of defined function identifiers in this context
    pub fn known_functions_mut(&mut self) -> &mut HashSet<String> {
        &mut self.known_functions
    }

    /// Get the list of defined templates in this context
    pub fn declared_templates(&self) -> &HashMap<String, TemplateDefinition> {
        &self.declared_templates
    }

    /// Get the list of defined pointer types in this context
    pub fn declared_pointer_types(&self) -> &HashMap<String, FunctionPrototype> {
        &self.declared_pointer_types
    }

    /// Parse a top-level declaration from a GLSLT shader.
    ///
    /// If the declaration is a GLSLT definition, it will not be returned and stored as part of the
    /// context for future template instantiations.
    ///
    /// # Parameters
    ///
    /// * `extdecl`: declaration to parse
    ///
    /// # Returns
    ///
    /// `Ok(None)` if the declaration was parsed as a template or GLSLT definition. `Ok(Some(...))`
    /// if this declaration is not a template or needs to be instantiated in a context
    pub fn parse_external_declaration(
        &mut self,
        extdecl: ExternalDeclaration,
    ) -> Result<Option<ExternalDeclaration>> {
        match extdecl {
            ExternalDeclaration::Declaration(decl) => self.parse_declaration(decl),
            ExternalDeclaration::FunctionDefinition(def) => Ok(self
                .parse_function_definition(def)?
                .map(ExternalDeclaration::FunctionDefinition)),
            // Just forward the others
            other => Ok(Some(other)),
        }
    }

    /// Register a function name within the context.
    ///
    /// This is required until a proper symbol table is added in order to differentiate variables
    /// from function names when instantiating templates.
    ///
    /// # Parameters
    ///
    /// * `def`: function definition to register
    pub fn push_function_declaration(&mut self, def: &FunctionDefinition) {
        // We discovered a new function
        self.known_functions.insert(def.prototype.name.0.clone());
    }
}
