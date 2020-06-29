use std::collections::{HashMap, HashSet};

use glsl::syntax::*;
use glsl::visitor::*;

use crate::{Error, Result};

struct TemplateParameter {
    /// Name of the function pointer type
    typename: String,
    /// Template variable name
    symbol: Option<String>,
    /// Original parameter index
    index: usize,
}

pub struct TemplateDefinition {
    /// AST for the partially instantiated template definition.
    ///
    /// This has to be cloned and visited to replace the template parameters.
    ast: FunctionDefinition,
    /// List of template parameters
    parameters: Vec<TemplateParameter>,
}

fn arg_instantiate(tgt: &mut Expr, source_parameters: &Vec<Expr>) {
    // Declare the visitor for the substitution
    struct V<'s> {
        subs: HashMap<String, &'s Expr>,
    }

    impl Visitor for V<'_> {
        fn visit_expr(&mut self, e: &mut Expr) -> Visit {
            match e {
                Expr::Variable(ident) => {
                    if let Some(repl) = self.subs.get(ident.0.as_str()) {
                        *e = (*repl).clone();
                    }
                }
                _ => {}
            }

            Visit::Children
        }
    }

    // Perform substitutions
    let mut subs = HashMap::new();
    for (id, value) in source_parameters.iter().enumerate() {
        subs.insert(format!("_{}", id + 1), value);
    }

    tgt.visit(&mut V { subs });
}

fn expr_vec_to_id(exprs: &Vec<Expr>) -> String {
    let mut sbuf = String::new();

    // Transpile all expressions into the string buffer
    for expr in exprs {
        glsl::transpiler::glsl::show_expr(
            &mut sbuf,
            expr,
            &mut glsl::transpiler::glsl::FormattingState::default(),
        )
        .unwrap();
    }

    // Compute it's SHA-1
    use crypto::digest::Digest;
    use crypto::sha1::Sha1;

    let mut hasher = Sha1::new();
    hasher.input_str(&sbuf);

    hasher.result_str()[0..6].to_string()
}

impl TemplateDefinition {
    pub fn generate_id(&self, args: &Vec<Expr>) -> String {
        let args_id = expr_vec_to_id(&args);
        ["_", self.ast.prototype.name.0.as_str(), &args_id].join("_")
    }

    pub fn instantiate(
        &self,
        name: &str,
        parameters: &Vec<Expr>,
        known_functions: &HashSet<String>,
    ) -> FunctionDefinition {
        // Clone the AST
        let mut ast = self.ast.clone();

        // Declare the visitor for the substitution
        struct V<'s> {
            subs: HashMap<&'s str, &'s Expr>,
            known_functions: &'s HashSet<String>,
        }

        impl Visitor for V<'_> {
            fn visit_expr(&mut self, e: &mut Expr) -> Visit {
                match e {
                    Expr::FunCall(fun, src_args) => {
                        // Only consider raw identifiers for function names
                        if let FunIdentifier::Identifier(ident) = fun {
                            if let Some(arg) = self.subs.get(ident.0.as_str()) {
                                // This is the name of a function to be templated

                                // If the substitution is a function name, just replace it and pass
                                // argument as-is.
                                //
                                // Else, replace the entire function call with the templated
                                // expression
                                match arg {
                                    Expr::Variable(arg_ident)
                                        if self.known_functions.contains(arg_ident.0.as_str()) =>
                                    {
                                        ident.0 = arg_ident.0.clone();
                                    }
                                    other => {
                                        let mut res = (*other).clone();
                                        arg_instantiate(&mut res, &src_args);
                                        *e = res;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                Visit::Children
            }
        }

        // Perform substitutions
        let mut subs = HashMap::new();
        for (param, value) in self.parameters.iter().zip(parameters.iter()) {
            if let Some(ps) = &param.symbol {
                subs.insert(ps.as_str(), value);
            }
        }

        ast.visit(&mut V {
            subs,
            known_functions,
        });

        // Change the name
        ast.prototype.name.0 = name.to_string();

        ast
    }

    pub fn extract_template_parameters(&self, args: &mut Vec<Expr>) -> Result<Vec<Expr>> {
        let mut idx = 0;
        let mut it = self.parameters.iter();
        let mut current = it.next();

        let (res, other): (Vec<_>, Vec<_>) = args.drain(0..args.len()).partition(|_| {
            let res = if let Some(c) = current {
                if c.index == idx {
                    true
                } else if c.index < idx {
                    current = it.next();
                    c.index == idx
                } else {
                    false
                }
            } else {
                false
            };

            idx += 1;
            res
        });

        // Put regular args back into the function call
        args.extend(other.into_iter());

        // Return template args with their values
        Ok(res)
    }
}

pub enum TryTemplate {
    Template(TemplateDefinition),
    Function(FunctionDefinition),
}

pub fn parse_definition_as_template(
    mut def: FunctionDefinition,
    declared_pointer_types: &HashMap<String, FunctionPrototype>,
) -> Result<TryTemplate> {
    let mut parameters = Vec::new();
    let mut non_template_parameters = Vec::new();

    for (arg_id, parameter) in def
        .prototype
        .parameters
        .drain(0..def.prototype.parameters.len())
        .enumerate()
    {
        let (n, t) = match &parameter {
            FunctionParameterDeclaration::Named(_, d) => (Some(d.ident.ident.0.clone()), &d.ty),
            FunctionParameterDeclaration::Unnamed(_, t) => (None, t),
        };

        if let TypeSpecifierNonArray::TypeName(tn) = &t.ty {
            if declared_pointer_types.contains_key(&tn.0) {
                if t.array_specifier.is_some() {
                    return Err(Error::ArrayedTemplateParameter {
                        name: def.prototype.name.to_string(),
                        index: arg_id,
                    });
                } else {
                    parameters.push(TemplateParameter {
                        typename: tn.0.clone(),
                        symbol: n,
                        index: arg_id,
                    });

                    continue;
                }
            }
        }

        non_template_parameters.push(parameter);
    }

    def.prototype
        .parameters
        .extend(non_template_parameters.into_iter());

    if parameters.is_empty() {
        Ok(TryTemplate::Function(def))
    } else {
        Ok(TryTemplate::Template(TemplateDefinition {
            ast: def,
            parameters,
        }))
    }
}
