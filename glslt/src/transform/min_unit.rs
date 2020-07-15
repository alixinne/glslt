use std::collections::{HashMap, HashSet};

use glsl::syntax::*;
use glsl::visitor::*;

use super::instantiate::InstantiateTemplate;
use super::{Context, TransformUnit};

use crate::{Error, Result};

mod dependency_dag;
use dependency_dag::*;

/// Minimizing GLSLT template instantiation context
#[derive(Default, Debug, Clone)]
pub struct MinUnit {
    /// Template definition context
    ctx: Context,
    /// External declaration repository
    external_declarations: HashMap<ExternalIdentifier, ExternalDeclaration>,
    /// Dependency graph, built as declarations are added to this unit
    dag: DependencyDag,
    /// Static contents that can be included before other declarations (such as #version, precision
    /// qualifiers, etc.)
    static_declarations: Vec<ExternalDeclaration>,
}

impl MinUnit {
    /// Create a new minifying transform unit
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new minifying transform unit using pre-defined templates
    ///
    /// # Parameters
    ///
    /// * `ctx`: context to pull pre-defined templates from
    pub fn with_context(ctx: Context) -> Self {
        Self {
            ctx,
            ..Default::default()
        }
    }

    /// Obtain an iterator to the functions defined in the current unit
    pub fn iter_functions(&self) -> impl Iterator<Item = &'_ FunctionDefinition> {
        self.external_declarations
            .values()
            .filter_map(|ed| match ed {
                ExternalDeclaration::FunctionDefinition(fd) => Some(fd),
                _ => None,
            })
    }

    /// Transform this unit into a GLSL syntax tree
    ///
    /// Only the functions included in `wanted` and their transitive dependencies will be included
    /// in the output.
    ///
    /// # Parameters
    ///
    /// * `wanted`: list of function names to include in the result
    pub fn into_translation_unit<'a>(
        self,
        wanted: impl std::iter::Iterator<Item = &'a str>,
    ) -> Result<TranslationUnit> {
        // Resulting declarations
        let mut external_declarations = self.static_declarations;

        // TODO: Maybe wanted can include other declarations than functions?
        let wanted: Vec<_> = wanted
            .map(|id| ExternalIdentifier::FunctionDefinition(id.to_owned()))
            .collect();

        // Extract declarations
        let mut stored_decls = self.external_declarations;

        // Add dependencies from wanted into the result
        // TODO: filter_map just ignores missing dependencies, we should at least warn the user
        external_declarations.extend(
            self.dag
                .into_dependencies(&wanted)
                .into_iter()
                .filter_map(|id| stored_decls.remove(&id)),
        );

        Ok(TranslationUnit(
            NonEmpty::from_non_empty_iter(external_declarations.into_iter())
                .ok_or_else(|| Error::EmptyInput)?,
        ))
    }

    fn extend_dag(&mut self, tu: &mut impl Host) {
        struct VisitorData<'e> {
            current_scope_name: Option<usize>,
            this: &'e mut MinUnit,
        }

        impl Visitor for VisitorData<'_> {
            fn visit_external_declaration(&mut self, _node: &mut ExternalDeclaration) -> Visit {
                self.current_scope_name = None;

                Visit::Children
            }

            fn visit_struct_specifier(&mut self, node: &mut StructSpecifier) -> Visit {
                if self.current_scope_name.is_none() {
                    if let Some(name) = &node.name {
                        self.current_scope_name = Some(
                            self.this
                                .dag
                                .declare_symbol(ExternalId::Declaration(name.0.as_str())),
                        );
                    }
                }

                Visit::Children
            }

            fn visit_function_definition(&mut self, node: &mut FunctionDefinition) -> Visit {
                self.current_scope_name = Some(self.this.dag.declare_symbol(
                    ExternalId::FunctionDefinition(node.prototype.name.0.as_str()),
                ));

                Visit::Children
            }

            fn visit_type_name(&mut self, node: &mut TypeName) -> Visit {
                if let Some(csn) = self.current_scope_name {
                    let this = self
                        .this
                        .dag
                        .declare_symbol(ExternalId::Declaration(node.0.as_str()));

                    if this != csn {
                        self.this.dag.add_dep(csn, this);
                    }
                }

                Visit::Children
            }

            fn visit_fun_identifier(&mut self, node: &mut FunIdentifier) -> Visit {
                if let FunIdentifier::Identifier(ident) = node {
                    if let Some(csn) = self.current_scope_name {
                        let this = self
                            .this
                            .dag
                            .declare_symbol(ExternalId::FunctionDefinition(ident.0.as_str()));
                        self.this.dag.add_dep(csn, this);
                    }
                }

                Visit::Children
            }

            fn visit_identifier(&mut self, node: &mut Identifier) -> Visit {
                if self
                    .this
                    .external_declarations
                    .contains_key(&ExternalIdentifier::Declaration(node.0.clone()))
                {
                    if let Some(csn) = self.current_scope_name {
                        // TODO: Declaration?
                        let this = self
                            .this
                            .dag
                            .declare_symbol(ExternalId::Declaration(node.0.as_str()));
                        self.this.dag.add_dep(csn, this);
                    }
                }

                Visit::Children
            }
        }

        let mut visitor = VisitorData {
            current_scope_name: None,
            this: self,
        };

        tu.visit(&mut visitor);
    }
}

impl TransformUnit for MinUnit {
    fn ctx(&self) -> &Context {
        &self.ctx
    }

    fn known_functions(&self) -> &HashSet<String> {
        self.ctx.known_functions()
    }

    fn template_instance_declared(&self, template_name: &str) -> bool {
        self.external_declarations
            .contains_key(&ExternalIdentifier::FunctionDefinition(
                template_name.to_owned(),
            ))
    }

    fn register_template_instance<'s>(
        &mut self,
        _template_name: &str,
        instance: FunctionDefinition,
    ) {
        self.push_function_declaration(instance);
    }

    fn push_function_declaration(&mut self, mut def: FunctionDefinition) {
        // Register the function as a known function
        self.ctx
            .known_functions_mut()
            .insert(def.prototype.name.0.clone());

        // Register it in the dependency graph
        self.extend_dag(&mut def);

        // Add the definition to the declarations
        self.external_declarations.insert(
            ExternalIdentifier::FunctionDefinition(def.prototype.name.0.clone()),
            ExternalDeclaration::FunctionDefinition(def),
        );
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
                other => match other {
                    ExternalDeclaration::FunctionDefinition(_) => {}
                    ExternalDeclaration::Preprocessor(ref pp) => match pp {
                        Preprocessor::Define(PreprocessorDefine::ObjectLike { ident, .. }) => {
                            self.external_declarations
                                .insert(ExternalIdentifier::Declaration(ident.0.clone()), other);
                        }
                        Preprocessor::Define(PreprocessorDefine::FunctionLike {
                            ident, ..
                        }) => {
                            self.external_declarations.insert(
                                ExternalIdentifier::FunctionDefinition(ident.0.clone()),
                                other,
                            );
                        }
                        Preprocessor::Version(_) | Preprocessor::Extension(_) => {
                            self.static_declarations.push(other);
                        }
                        rest => return Err(Error::UnsupportedPreprocessor(rest.clone())),
                    },
                    ExternalDeclaration::Declaration(ref decl) => match decl {
                        Declaration::FunctionPrototype(_) => {
                            unreachable!("prototype already consumed by template engine")
                        }
                        Declaration::InitDeclaratorList(idl) => {
                            // TODO: Handle variable declarations at top-level using
                            // InitDeclaratorList. For now, this only handles struct declarations.
                            if let TypeSpecifierNonArray::Struct(ss) = &idl.head.ty.ty.ty {
                                // It's a struct declaration
                                if let Some(tn) = &ss.name {
                                    self.external_declarations.insert(
                                        ExternalIdentifier::Declaration(tn.0.clone()),
                                        other,
                                    );
                                } else {
                                    return Err(Error::UnsupportedIDL(idl.clone()));
                                }
                            } else {
                                // It's a variable declaration, maybe?
                                if let Some(name) = &idl.head.name {
                                    self.external_declarations.insert(
                                        ExternalIdentifier::Declaration(name.0.clone()),
                                        other,
                                    );
                                } else {
                                    return Err(Error::UnsupportedIDL(idl.clone()));
                                }
                            }
                        }
                        Declaration::Precision(_, _) | Declaration::Block(_) => {
                            // TODO: How to handle Declaration::Block?
                            self.static_declarations.push(other);
                        }
                        Declaration::Global(tq, identifiers) => {
                            // TODO: How are globals used by function code?
                            for id in identifiers {
                                self.external_declarations.insert(
                                    ExternalIdentifier::Declaration(id.0.clone()),
                                    ExternalDeclaration::Declaration(Declaration::Global(
                                        tq.clone(),
                                        vec![id.clone()],
                                    )),
                                );
                            }
                        }
                    },
                },
            }
        }

        Ok(())
    }
}
