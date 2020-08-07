//! Definition of the local scope structure for template instantiation

use std::collections::HashMap;

use glsl::syntax::*;
use glsl::visitor::{Host, Visit, Visitor};

use itertools::Itertools;

use super::template::TemplateDefinition;

/// A local scope for resolving template arguments inside a template function
#[derive(Default, Debug, Clone)]
pub struct LocalScope {
    /// Name of the current template scope
    name: String,
    /// List of ordered template parameters
    template_parameters: Vec<Expr>,
    /// Lookup table for template parameters by name
    template_parameters_by_name: HashMap<String, usize>,
    /// List of parameter names captured by entering the current scope
    captured_parameters: Vec<String>,
}

impl LocalScope {
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
    ) -> crate::Result<Self> {
        // Extract template parameters for this scope
        let mut template_parameters = template.extract_template_parameters(args)?;

        // Build lookup table
        let mut template_parameters_by_name = HashMap::with_capacity(template_parameters.len());
        for parameter in template.parameters() {
            if let Some(name) = parameter.symbol.as_ref() {
                template_parameters_by_name.insert(name.to_owned(), parameter.index);
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
}
