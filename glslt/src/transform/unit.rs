use std::sync::Arc;

use glsl_lang::ast::*;

use super::instantiate::InstantiateTemplate;
use super::{FnHandle, FnRef, GlobalScope, ParsedDeclaration, TransformUnit};

use crate::{Result, TransformConfig};

/// GLSLT template instantiation context
#[derive(Default, Debug, Clone)]
pub struct Unit {
    /// Template definition context
    global_scope: GlobalScope,
    /// Result of external declarations copied from input and generated through instantiation
    external_declarations: Vec<ExternalDeclaration>,
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

    /// Create a new transform unit with the given configuration
    ///
    /// # Parameters
    ///
    /// * `config`: transform configuration
    pub fn with_config(config: TransformConfig) -> Self {
        Self {
            global_scope: GlobalScope::with_config(config),
            external_declarations: Vec::new(),
        }
    }

    /// Transform this unit into a GLSL syntax tree
    ///
    /// All the declarations given as input will be included in-order in the output, with the
    /// template instantiations interleaved when needed.
    pub fn into_translation_unit(self) -> Result<TranslationUnit> {
        Ok(TranslationUnit(
            self.external_declarations.into_iter().collect(),
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

    fn push_function_declaration(&mut self, def: FunctionDefinition) {
        self.global_scope
            .known_functions_mut()
            .insert(def.prototype.name.0.clone(), def.prototype.clone());

        // Add the definition to the declarations
        // TODO: Don't clone def.span?
        let span = def.span;
        self.external_declarations.push(ExternalDeclaration::new(
            ExternalDeclarationData::FunctionDefinition(def),
            span,
        ));
    }

    fn parse_external_declaration(
        &mut self,
        extdecl: ExternalDeclaration,
    ) -> Result<Option<FnHandle>> {
        let unparsed = match self.global_scope.parse_external_declaration(extdecl)? {
            ParsedDeclaration::ConsumedAsType => {
                return Ok(None);
            }
            ParsedDeclaration::ConsumedAsTemplate(r) => {
                return Ok(Some(r.into()));
            }
            ParsedDeclaration::Unparsed(extdecl) => extdecl,
        };

        let extdecl = Arc::try_unwrap(unparsed).unwrap();
        match extdecl.content {
            ExternalDeclarationData::FunctionDefinition(def) => {
                // No template parameter, it's a "regular" function so it has to be
                // processed to instantiate parameters
                let decls =
                    InstantiateTemplate::new(None).instantiate(&mut self.global_scope, def)?;

                for d in decls {
                    self.push_function_declaration(d);
                }

                let f = self.external_declarations.last().unwrap();
                match &f.content {
                    ExternalDeclarationData::FunctionDefinition(def) => {
                        return Ok(Some(
                            Node::new(
                                FnRef {
                                    prototype: &def.prototype,
                                    statement: &def.statement,
                                },
                                extdecl.span,
                            )
                            .into(),
                        ));
                    }
                    _ => unreachable!(),
                }
            }
            other => self
                .external_declarations
                .push(Node::new(other, extdecl.span)),
        }

        Ok(None)
    }
}
