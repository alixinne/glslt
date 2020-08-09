use glsl::syntax::*;

use super::instantiate::InstantiateTemplate;
use super::{GlobalScope, TransformUnit};

use crate::{Error, Result};

/// GLSLT template instantiation context
#[derive(Default, Debug, Clone)]
pub struct Unit {
    /// Template definition context
    global_scope: GlobalScope,
    /// Result of external declarations copied from input and generated through instantiation
    external_declarations: Vec<Node<ExternalDeclaration>>,
}

impl Unit {
    /// Create a new transform unit
    pub fn new() -> Self {
        Self {
            global_scope: GlobalScope::new(),
            external_declarations: Vec::new(),
        }
    }

    /// Create a new transform unit using pre-defined templates
    ///
    /// # Parameters
    ///
    /// * `global_scope`: context to pull pre-defined templates from
    pub fn with_context(global_scope: GlobalScope) -> Self {
        Self {
            global_scope,
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
    fn global_scope(&self) -> &GlobalScope {
        &self.global_scope
    }

    fn global_scope_mut(&mut self) -> &mut GlobalScope {
        &mut self.global_scope
    }

    fn push_function_declaration(&mut self, def: Node<FunctionDefinition>) {
        self.global_scope
            .known_functions_mut()
            .insert(def.prototype.name.0.clone(), def.prototype.clone());

        // Add the definition to the declarations
        self.external_declarations
            .push(def.map(ExternalDeclaration::FunctionDefinition));
    }

    fn parse_external_declaration(&mut self, extdecl: Node<ExternalDeclaration>) -> Result<()> {
        if let Some(extdecl) = self.global_scope.parse_external_declaration(extdecl)? {
            match extdecl.contents {
                ExternalDeclaration::FunctionDefinition(def) => {
                    // No template parameter, it's a "regular" function so it has to be
                    // processed to instantiate parameters
                    let decls = InstantiateTemplate::new()
                        .instantiate(&mut self.global_scope, Node::new(def, extdecl.span_id))?;

                    for d in decls {
                        self.push_function_declaration(d);
                    }
                }
                other => self
                    .external_declarations
                    .push(Node::new(other, extdecl.span_id)),
            }
        }

        Ok(())
    }
}
