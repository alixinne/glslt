use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use glsl::syntax::*;

use super::template::{TemplateDefinition, TryTemplate};
use crate::{Error, Result};

/// GLSLT template definition global scope
#[derive(Default, Debug, Clone)]
pub struct GlobalScope {
    /// Known pointer types
    declared_pointer_types: HashMap<String, FunctionPrototype>,
    /// Known GLSLT template functions
    declared_templates: HashMap<String, Rc<TemplateDefinition>>,
    /// Identifiers of function declarations
    known_functions: HashSet<String>,
}

impl GlobalScope {
    /// Create a new template definition global scope
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
        def: Node<FunctionDefinition>,
    ) -> Result<Option<Node<FunctionDefinition>>> {
        // A function definition is a template if any of its arguments is a pointer
        let name = def.prototype.name.0.clone();
        let template =
            super::template::parse_definition_as_template(def, &self.declared_pointer_types)?;

        match template {
            TryTemplate::Template(template) => {
                info!("declared template: {}", template.ast.prototype.name.0);

                // We found a template parameter, so it's a template function
                self.declared_templates.insert(name, Rc::new(template));
                Ok(None)
            }
            TryTemplate::Function(def) => Ok(Some(def)),
        }
    }

    /// Get the list of defined function identifiers in this global scope
    pub fn known_functions(&self) -> &HashSet<String> {
        &self.known_functions
    }

    /// Get the list of defined function identifiers in this global scope
    pub fn known_functions_mut(&mut self) -> &mut HashSet<String> {
        &mut self.known_functions
    }

    /// Get the template corresponding to the given name
    pub fn get_template(&self, template_name: &str) -> Option<Rc<TemplateDefinition>> {
        self.declared_templates
            .get(template_name)
            .map(|v| v.clone())
    }

    /// Get the list of defined templates in this global scope
    pub fn declared_templates(&self) -> &HashMap<String, Rc<TemplateDefinition>> {
        &self.declared_templates
    }

    /// Get the list of defined pointer types in this global scope
    pub fn declared_pointer_types(&self) -> &HashMap<String, FunctionPrototype> {
        &self.declared_pointer_types
    }

    /// Parse a top-level declaration from a GLSLT shader.
    ///
    /// If the declaration is a GLSLT definition, it will not be returned and stored as part of the
    /// global scope for future template instantiations.
    ///
    /// # Parameters
    ///
    /// * `extdecl`: declaration to parse
    ///
    /// # Returns
    ///
    /// `Ok(None)` if the declaration was parsed as a template or GLSLT definition. `Ok(Some(...))`
    /// if this declaration is not a template or needs to be instantiated in a global scope
    pub fn parse_external_declaration(
        &mut self,
        extdecl: Node<ExternalDeclaration>,
    ) -> Result<Option<Node<ExternalDeclaration>>> {
        let span_id = extdecl.span_id;

        match extdecl.contents {
            ExternalDeclaration::Declaration(decl) => self
                .parse_declaration(decl)
                .map(|ed| ed.map(|ed| Node::new(ed, span_id))),
            ExternalDeclaration::FunctionDefinition(def) => Ok(self
                .parse_function_definition(Node::new(def, span_id))?
                .map(|n| {
                    Node::new(
                        ExternalDeclaration::FunctionDefinition(n.into_inner()),
                        span_id,
                    )
                })),
            // Just forward the others
            other => Ok(Some(Node::new(other, span_id))),
        }
    }

    /// Register a function name within the global scope.
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
