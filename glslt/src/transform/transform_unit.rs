use std::collections::HashMap;

use glsl::syntax::*;

use crate::Result;

pub trait TransformUnit<'c> {
    fn push_function_declaration(&mut self, def: FunctionDefinition);

    fn transform_call(
        &mut self,
        fun: &mut Identifier,
        args: &mut Vec<Expr>,
        symbol_table: &HashMap<String, super::instantiate::DeclaredSymbol>,
    ) -> Result<()>;
}
