use std::collections::HashMap;

use glsl::syntax::*;
use glsl::visitor::*;

use crate::{Error, Result};

use super::TransformUnit;

#[derive(Debug)]
pub struct DeclaredSymbol {
    pub symbol_id: usize,
    pub gen_id: Identifier,
    pub decl_type: TypeSpecifier,
    pub array: Option<ArraySpecifier>,
}

pub struct InstantiateTemplate<'c, 'd> {
    ctx: &'c mut dyn TransformUnit<'d>,
    error: Option<Error>,
    symbol_table: HashMap<String, DeclaredSymbol>,
}

impl<'c, 'd> InstantiateTemplate<'c, 'd> {
    pub fn new(ctx: &'c mut dyn TransformUnit<'d>) -> Self {
        Self {
            ctx,
            error: None,
            symbol_table: HashMap::new(),
        }
    }

    pub fn instantiate(mut self, mut def: FunctionDefinition) -> Result<()> {
        // Transform definition. The visitor is responsible for instantiating templates
        def.visit(&mut self);

        if let Some(error) = self.error.take() {
            return Err(error);
        }

        self.ctx.push_function_declaration(def);

        Ok(())
    }

    fn new_gen_id(&self) -> Identifier {
        Identifier::new(format!("_glslt_lp{}", self.symbol_table.len())).unwrap()
    }
}

impl Visitor for InstantiateTemplate<'_, '_> {
    fn visit_function_parameter_declarator(
        &mut self,
        p: &mut FunctionParameterDeclarator,
    ) -> Visit {
        // Register a declared parameter
        self.symbol_table.insert(
            p.ident.ident.0.clone(),
            DeclaredSymbol {
                symbol_id: self.symbol_table.len(),
                gen_id: self.new_gen_id(),
                decl_type: p.ty.clone(),
                array: p.ident.array_spec.clone(),
            },
        );

        Visit::Children
    }

    fn visit_init_declarator_list(&mut self, idl: &mut InitDeclaratorList) -> Visit {
        // Register all declared variables
        self.symbol_table.insert(
            idl.head.name.as_ref().unwrap().0.clone(),
            DeclaredSymbol {
                symbol_id: self.symbol_table.len(),
                gen_id: self.new_gen_id(),
                decl_type: idl.head.ty.ty.clone(),
                array: idl.head.array_specifier.clone(),
            },
        );

        // Add tail
        for t in &idl.tail {
            self.symbol_table.insert(
                t.ident.ident.0.clone(),
                DeclaredSymbol {
                    symbol_id: self.symbol_table.len(),
                    gen_id: self.new_gen_id(),
                    decl_type: idl.head.ty.ty.clone(),
                    array: idl.head.array_specifier.clone(),
                },
            );
        }

        Visit::Children
    }

    fn visit_expr(&mut self, e: &mut Expr) -> Visit {
        match e {
            Expr::FunCall(fun, args) => {
                // Only consider raw identifiers for function names
                if let FunIdentifier::Identifier(ident) = fun {
                    if let Err(error) = self.ctx.transform_call(ident, args, &self.symbol_table) {
                        self.error = Some(error);
                    }
                }
            }
            _ => {}
        }

        Visit::Children
    }
}
