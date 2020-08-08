//! Definition of the local scope structure for template instantiation

use std::collections::HashMap;

use glsl::syntax::*;
use glsl::visitor::{Host, Visit, Visitor};

use itertools::Itertools;

use super::template::TemplateDefinition;
use super::{instantiate::InstantiateTemplate, Scope};

/// A local scope for resolving template arguments inside a template function
#[derive(Debug)]
pub struct LocalScope<'p> {
    /// Parent scope reference
    parent: &'p mut dyn Scope,
    /// Name of the current template scope
    name: String,
    /// List of ordered template parameters
    template_parameters: Vec<Expr>,
    /// Lookup table for template parameters by name
    template_parameters_by_name: HashMap<String, usize>,
    /// List of parameter names captured by entering the current scope
    captured_parameters: Vec<String>,
}

impl<'p> LocalScope<'p> {
    /// Enter a new local instantiation scope
    ///
    /// # Parameters
    ///
    /// * `template`: template being instantiated as part of a function call
    /// * `args`: list of expressions which are arguments to the template call
    /// * `symbol_table`: locally-declared symbols from the outer function
    pub fn new(
        template: &TemplateDefinition,
        args: &mut Vec<Expr>,
        symbol_table: &HashMap<String, super::instantiate::DeclaredSymbol>,
        parent: &'p mut dyn Scope,
    ) -> crate::Result<Self> {
        // Extract template parameters for this scope
        let mut template_parameters = template.extract_template_parameters(args)?;

        // Build lookup table
        let mut template_parameters_by_name = HashMap::with_capacity(template_parameters.len());
        for (index, parameter) in template.parameters().iter().enumerate() {
            if let Some(name) = parameter.symbol.as_ref() {
                template_parameters_by_name.insert(name.to_owned(), index);
            }
        }

        // Generate scope name
        let name = template.generate_id(&template_parameters[..]);

        // Extract the set of captured variables
        struct Capturer<'ds> {
            st: &'ds HashMap<String, super::instantiate::DeclaredSymbol>,
            captured: HashMap<String, &'ds super::instantiate::DeclaredSymbol>,
        }

        impl Visitor for Capturer<'_> {
            fn visit_expr(&mut self, e: &mut Expr) -> Visit {
                if let Expr::Variable(ident) = e {
                    // This is a variable. If it's in the symbol table, it needs to be
                    // captured and renamed

                    if let Some(sb) = self.st.get(ident.0.as_str()) {
                        // Add it to the list of captured variables
                        self.captured.insert(ident.0.clone(), sb);
                        // Rename the reference
                        *ident = sb.gen_id.clone();
                    }
                }

                Visit::Children
            }
        }

        // Visit the input expressions
        let mut capturer = Capturer {
            st: symbol_table,
            captured: HashMap::new(),
        };

        for tp in &mut template_parameters {
            tp.visit(&mut capturer);
        }

        // Extract the list of captured variables ordered by symbol_id
        let extra_parameters: Vec<_> = capturer
            .captured
            .into_iter()
            .sorted_by_key(|ep| ep.1.symbol_id)
            .map(|(key, _)| key)
            .collect();

        Ok(Self {
            parent,
            name,
            template_parameters,
            template_parameters_by_name,
            captured_parameters: extra_parameters,
        })
    }

    /// Get the name of the current template scope
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the list of template parameters
    pub fn template_parameters(&self) -> &[Expr] {
        &self.template_parameters[..]
    }

    /// Get the list of captured parameter names
    pub fn captured_parameters(&self) -> &[String] {
        &self.captured_parameters[..]
    }

    /// Transform the target function call expression into a GLSL function call
    ///
    /// This takes an exclusive reference to the expression to modify it, but assumes it's a
    /// function call and will panic if it isn't the case.
    pub fn transform_fn_call(
        &mut self,
        e: &mut Expr,
        instantiator: &mut InstantiateTemplate,
        template: &TemplateDefinition,
    ) -> crate::Result<()> {
        match e {
            Expr::FunCall(fun, src_args) => {
                // Only consider raw identifiers for function names
                if let FunIdentifier::Identifier(ident) = fun {
                    if let Some((arg_id, arg)) = self
                        .template_parameters_by_name
                        .get(ident.0.as_str())
                        .map(|id| (id, &self.template_parameters[*id]))
                    {
                        // If the substitution is a function name, just replace it and pass
                        // argument as-is.
                        //
                        // Else, replace the entire function call with the templated
                        // expression
                        match arg {
                            Expr::Variable(arg_ident) => {
                                if let Some(target) =
                                    self.resolve_function_name(arg_ident.0.as_str())
                                {
                                    debug!(
                                        "in {}: resolved raw function name {:?}",
                                        self.name, arg_ident
                                    );

                                    ident.0 = target;
                                } else {
                                    debug!(
                                        "in {}: unresolved raw function name {:?}, treating as lambda",
                                        self.name,
                                        arg_ident
                                    );
                                    let mut res = arg.clone();
                                    lambda_instantiate(
                                        &mut res,
                                        &src_args,
                                        &self
                                            .declared_pointer_types()
                                            .get(template.parameters()[*arg_id].typename.as_str())
                                            .unwrap(),
                                    );

                                    *e = res;
                                }
                            }
                            other => {
                                debug!("in {}: lambda expression: {:?}", self.name, other);
                                let mut res = other.clone();
                                lambda_instantiate(
                                    &mut res,
                                    &src_args,
                                    &self
                                        .declared_pointer_types()
                                        .get(template.parameters()[*arg_id].typename.as_str())
                                        .unwrap(),
                                );

                                *e = res;
                            }
                        }
                    } else {
                        debug!(
                            "in {}: found nested template call to {:?}({:?})",
                            self.name, ident, src_args
                        );
                        instantiator.visit_fun_call(fun, src_args, self as &mut dyn Scope);
                    }
                } else {
                    warn!(
                        "in {}: unsupported function identifier: {:?}",
                        self.name, fun
                    );
                }
            }
            other => {
                panic!("LocalScope::transform_fn_call can only process function call expressions, got {:?}", other);
            }
        }

        Ok(())
    }
}

impl Scope for LocalScope<'_> {
    fn parent_scope(&self) -> Option<&dyn Scope> {
        Some(self.parent)
    }

    fn declared_pointer_types(&self) -> &HashMap<String, FunctionPrototype> {
        self.parent.declared_pointer_types()
    }

    fn get_template(&self, template_name: &str) -> Option<std::rc::Rc<TemplateDefinition>> {
        self.parent.get_template(template_name)
    }

    fn template_instance_declared(&self, template_name: &str) -> bool {
        self.parent.template_instance_declared(template_name)
    }

    fn register_template_instance(
        &mut self,
        template_name: &str,
        instance: Node<FunctionDefinition>,
    ) {
        self.parent
            .register_template_instance(template_name, instance)
    }

    fn take_instanced_templates(&mut self) -> Vec<Node<FunctionDefinition>> {
        self.parent.take_instanced_templates()
    }

    fn resolve_function_name(&self, name: &str) -> Option<String> {
        // Look at the local scope arguments for a definition
        if let Some(arg) = self
            .template_parameters_by_name
            .get(name)
            .and_then(|id| self.template_parameters.get(*id))
        {
            match arg {
                Expr::Variable(ident) => {
                    // This may be a name in the parent scope, delegate to parent scope
                    self.parent.resolve_function_name(ident.0.as_str())
                }
                // This is a lambda expression, we can't resolve it to a function name
                _ => None,
            }
        } else {
            // No argument matching this, delegate to parent scope
            self.parent.resolve_function_name(name)
        }
    }
}

fn lambda_instantiate(tgt: &mut Expr, source_parameters: &[Expr], prototype: &FunctionPrototype) {
    // Declare the visitor for the substitution
    struct V<'s> {
        subs: HashMap<String, &'s Expr>,
    }

    impl Visitor for V<'_> {
        fn visit_expr(&mut self, e: &mut Expr) -> Visit {
            if let Expr::Variable(ident) = e {
                if let Some(repl) = self.subs.get(ident.0.as_str()) {
                    *e = (*repl).clone();
                }
            }

            Visit::Children
        }
    }

    // Perform substitutions
    let mut subs = HashMap::new();
    for (id, value) in source_parameters.iter().enumerate() {
        subs.insert(format!("_{}", id + 1), value);

        if let FunctionParameterDeclaration::Named(_, p) = &prototype.parameters[id] {
            subs.insert(format!("_{}", p.ident.ident.0), value);
        }
    }

    tgt.visit(&mut V { subs });
}
