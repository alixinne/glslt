use glsl::syntax::*;
use glsl::visitor::*;

use crate::{Error, Result};

use super::context::Context;

pub struct InstantiateTemplate<'c> {
    ctx: &'c mut Context,
    error: Option<Error>,
}

impl<'c> InstantiateTemplate<'c> {
    pub fn new(ctx: &'c mut Context) -> Self {
        Self { ctx, error: None }
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
}

impl Visitor for InstantiateTemplate<'_> {
    fn visit_expr(&mut self, e: &mut Expr) -> Visit {
        match e {
            Expr::FunCall(fun, args) => {
                // Only consider raw identifiers for function names
                if let FunIdentifier::Identifier(ident) = fun {
                    if let Err(error) = self.ctx.transform_call(ident, args) {
                        self.error = Some(error);
                    }
                }
            }
            _ => {}
        }

        Visit::Children
    }
}
