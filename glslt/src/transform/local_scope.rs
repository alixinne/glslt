//! Definition of the local scope structure for template instantiation

use std::collections::HashMap;

use glsl::syntax::*;
use glsl::visitor::{Host, Visit, Visitor};

use itertools::Itertools;

use super::template::TemplateDefinition;
use super::{instantiate::InstantiateTemplate, ResolvedArgument, ResolvedArgumentExpr, Scope};

/// A local scope for resolving template arguments inside a template function
#[derive(Debug)]
pub struct LocalScope<'p, 'q> {
    /// Parent scope reference
    parent: &'p mut dyn Scope,
    /// Name of the current template scope
    name: String,
    /// List of ordered template parameters
    template_parameters: Vec<(Expr, &'q str)>,
    /// Lookup table for template parameters by name
    template_parameters_by_name: HashMap<String, usize>,
    /// List of parameter names captured by entering the current scope
    captured_parameters: Vec<String>,
}

impl<'p, 'q> LocalScope<'p, 'q> {
    /// Enter a new local instantiation scope
    ///
    /// # Parameters
    ///
    /// * `template`: template being instantiated as part of a function call
    /// * `args`: list of expressions which are arguments to the template call
    /// * `symbol_table`: locally-declared symbols from the outer function
    pub fn new(
        template: &'q TemplateDefinition,
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
            tp.0.visit(&mut capturer);
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
        self.transform_arg_call_typed(e, instantiator, &template.ast().prototype)
    }

    fn transform_arg_call_typed(
        &mut self,
        e: &mut Expr,
        instantiator: &mut InstantiateTemplate,
        prototype: &FunctionPrototype,
    ) -> crate::Result<()> {
        match e {
            Expr::FunCall(fun, src_args) => {
                // Only consider raw identifiers for function names
                if let FunIdentifier::Identifier(ident) = fun {
                    if let Some(arg) = self
                        .template_parameters_by_name
                        .get(ident.0.as_str())
                        .map(|id| &self.template_parameters[*id])
                    {
                        // If the substitution is a function name, just replace it and pass
                        // argument as-is.
                        //
                        // Else, replace the entire function call with the templated
                        // expression
                        let lambda_expr = match &arg.0 {
                            Expr::Variable(arg_ident) => {
                                if let Some(target) =
                                    self.resolve_function_name(arg_ident.0.as_str())
                                {
                                    match target.body {
                                        ResolvedArgumentExpr::FunctionName(name) => {
                                            debug!(
                                                "in {}: resolved raw function name {:?}",
                                                self.name, arg_ident
                                            );

                                            ident.0 = name;
                                            None // Transformation complete
                                        }
                                        ResolvedArgumentExpr::Lambda(expr) => {
                                            debug!(
                                                "in {}: resolved function name {:?} as outer lambda",
                                                self.name, arg_ident
                                            );

                                            Some(expr)
                                        }
                                    }
                                } else {
                                    debug!(
                                        "in {}: unresolved raw function name {:?}, treating as lambda",
                                        self.name,
                                        arg_ident
                                    );

                                    Some(arg.0.clone())
                                }
                            }
                            other => {
                                debug!("in {}: lambda expression: {:?}", self.name, other);
                                Some(other.clone())
                            }
                        };

                        if let Some(mut expr) = lambda_expr {
                            // Replace the parameters in the call
                            lambda_instantiate(&mut expr, &src_args, prototype);

                            // Replace lambda arguments in the generated expression
                            *e = expr;
                        }
                    } else {
                        debug!(
                            "in {}: found nested template call to {:?}({:?})",
                            self.name, ident, src_args
                        );

                        instantiator.visit_fun_call(e, self as &mut dyn Scope);
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

impl Scope for LocalScope<'_, '_> {
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

    fn register_template_instance(&mut self, definitions: Vec<Node<FunctionDefinition>>) {
        self.parent.register_template_instance(definitions)
    }

    fn take_instanced_templates(&mut self) -> Vec<Node<FunctionDefinition>> {
        self.parent.take_instanced_templates()
    }

    fn resolve_function_name(&self, name: &str) -> Option<ResolvedArgument> {
        // Look at the local scope arguments for a definition
        if let Some(arg) = self
            .template_parameters_by_name
            .get(name)
            .and_then(|id| self.template_parameters.get(*id))
        {
            match &arg.0 {
                Expr::Variable(ident) => {
                    // Only resolve to parent scope if this is the name of a parameter
                    if self
                        .template_parameters_by_name
                        .contains_key(ident.0.as_str())
                    {
                        debug!(
                            "in {}: resolve_function_name: asking parent for {}",
                            self.name, ident
                        );

                        // This is a name coming from the parent scope
                        self.parent.resolve_function_name(ident.0.as_str())
                    } else {
                        debug!(
                            "in {}: resolve_function_name: returning {} as lambda for {}",
                            self.name, ident, name
                        );

                        // This is not a name coming from the parent scope (i.e. parameter)
                        // This might be a global function (thus the parent resolve_function_name
                        // should return Some(...)) or a lambda with only one atom in its
                        // expression.
                        // TODO: Is this really the best way to resolve this ambiguity?

                        self.parent
                            .resolve_function_name(ident.0.as_str())
                            .or_else(|| {
                                self.declared_pointer_types()
                                    .get(arg.1)
                                    .map(|pointer_type| ResolvedArgument {
                                        body: ResolvedArgumentExpr::Lambda(arg.0.clone()),
                                        pointer_type,
                                    })
                            })
                    }
                }
                // This is a lambda expression, we can't resolve it to a function name
                // TODO: Propagate error
                other => self
                    .declared_pointer_types()
                    .get(arg.1)
                    .map(|pointer_type| ResolvedArgument {
                        body: ResolvedArgumentExpr::Lambda(other.clone()),
                        pointer_type,
                    }),
            }
        } else {
            // No argument matching this, delegate to parent scope
            self.parent.resolve_function_name(name)
        }
    }

    fn transform_arg_call(
        &mut self,
        expr: &mut Expr,
        instantiator: &mut InstantiateTemplate,
    ) -> crate::Result<()> {
        match expr {
            Expr::FunCall(FunIdentifier::Identifier(ident), _) => {
                if let Some(tplarg) = self
                    .template_parameters_by_name
                    .get(ident.0.as_str())
                    .and_then(|id| self.template_parameters.get(*id))
                {
                    // TODO: Remove this clone, with an Rc?
                    let c = self
                        .declared_pointer_types()
                        .get(tplarg.1)
                        .ok_or_else(|| crate::Error::UndeclaredPointerType(tplarg.1.to_owned()))?
                        .clone();

                    debug!("transforming call to {:?} using prototype {:?}", expr, c);
                    return self.transform_arg_call_typed(expr, instantiator, &c);
                }
            }
            _ => panic!("unsupported expression for LocalScope::transform_arg_call"),
        }

        Err(crate::Error::TransformAsTemplate)
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
