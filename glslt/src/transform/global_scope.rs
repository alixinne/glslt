use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use glsl::syntax::*;

use super::template::{TemplateDefinition, TryTemplate};
use super::{ResolvedArgument, ResolvedArgumentExpr, Scope};

use crate::{Error, Result};

/// GLSLT template definition global scope
#[derive(Default, Debug, Clone)]
pub struct GlobalScope {
    /// Known pointer types
    declared_pointer_types: HashMap<String, FunctionPrototype>,
    /// Known GLSLT template functions
    declared_templates: HashMap<String, Rc<TemplateDefinition>>,
    /// Identifiers of function declarations
    known_functions: HashMap<String, FunctionPrototype>,
    /// Identifiers of already instantiated templates
    instantiated_templates: HashSet<String>,
    /// Pending external declarations
    instanced_templates: Vec<Node<FunctionDefinition>>,
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
                info!("declared template: {}", template.ast().prototype.name.0);

                // We found a template parameter, so it's a template function
                self.declared_templates.insert(name, Rc::new(template));
                Ok(None)
            }
            TryTemplate::Function(def) => Ok(Some(def)),
        }
    }

    /// Get the list of defined function identifiers in this global scope
    pub fn known_functions_mut(&mut self) -> &mut HashMap<String, FunctionPrototype> {
        &mut self.known_functions
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
        self.known_functions
            .insert(def.prototype.name.0.clone(), def.prototype.clone());
    }
}

impl Scope for GlobalScope {
    fn parent_scope(&self) -> Option<&dyn Scope> {
        None
    }

    fn declared_pointer_types(&self) -> &HashMap<String, FunctionPrototype> {
        &self.declared_pointer_types
    }

    fn get_template(&self, template_name: &str) -> Option<Rc<TemplateDefinition>> {
        self.declared_templates.get(template_name).cloned()
    }

    fn template_instance_declared(&self, template_name: &str) -> bool {
        self.instantiated_templates.contains(template_name)
    }

    fn register_template_instance(&mut self, definitions: Vec<Node<FunctionDefinition>>) {
        for template in definitions {
            let template_name = template.prototype.name.0.as_str();

            // Take note we instantiated the template
            self.instantiated_templates.insert(template_name.to_owned());

            // Add them to the instanced templates
            self.instanced_templates.push(template);
        }
    }

    fn take_instanced_templates(&mut self) -> Vec<Node<FunctionDefinition>> {
        std::mem::replace(&mut self.instanced_templates, Vec::with_capacity(2))
    }

    fn resolve_function_name(&self, name: &str) -> Option<ResolvedArgument> {
        self.known_functions
            .get(name)
            .map(|proto| ResolvedArgument {
                body: ResolvedArgumentExpr::FunctionName(proto.name.0.clone()),
                pointer_type: proto,
            })
    }

    fn transform_arg_call(
        &mut self,
        _expr: &mut Expr,
        _instantiator: &mut super::instantiate::InstantiateTemplate,
    ) -> Result<()> {
        Err(Error::TransformAsTemplate)
    }
}
