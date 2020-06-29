use std::collections::{HashMap, HashSet};

use glsl::syntax::*;
use glsl::visitor::*;

use super::instantiate::InstantiateTemplate;
use super::template::{TemplateDefinition, TryTemplate};
use crate::{Error, Result};

#[derive(Default)]
pub struct Context {
    declared_pointer_types: HashMap<String, FunctionPrototype>,
    declared_templates: HashMap<String, TemplateDefinition>,
    external_declarations: Vec<ExternalDeclaration>,
    instantiated_templates: HashSet<String>,
    known_functions: HashSet<String>,
}

impl Context {
    fn parse_function_prototype(&mut self, prototype: FunctionPrototype) -> Result<()> {
        if let Some(previous) = self.declared_pointer_types.get(&prototype.name.0) {
            // Since function pointer types are raw identifiers, they have to be unique
            return Err(Error::new_duplicate_pointer_definition(
                &prototype.name,
                previous,
            ));
        } else {
            self.declared_pointer_types
                .insert(prototype.name.0.clone(), prototype);
        }

        Ok(())
    }

    fn parse_declaration(&mut self, decl: Declaration) -> Result<()> {
        match decl {
            Declaration::FunctionPrototype(prototype) => {
                // A function prototype is what we'll call a function pointer type
                self.parse_function_prototype(prototype)?;
            }
            other => self
                .external_declarations
                .push(ExternalDeclaration::Declaration(other)),
        }

        Ok(())
    }

    fn parse_function_definition(&mut self, def: FunctionDefinition) -> Result<()> {
        // A function definition is a template if any of its arguments is a pointer
        let name = def.prototype.name.0.clone();
        let template =
            super::template::parse_definition_as_template(def, &self.declared_pointer_types)?;

        match template {
            TryTemplate::Template(template) => {
                // We found a template parameter, so it's a template function
                self.declared_templates.insert(name, template);
            }
            TryTemplate::Function(def) => {
                // No template parameter, it's a "regular" function so it has to be
                // processed to instantiate parameters
                //
                // TODO: Recursive template instantiation?
                InstantiateTemplate::new(self).instantiate(def)?;
            }
        }

        Ok(())
    }

    pub fn parse_external_declaration(&mut self, extdecl: ExternalDeclaration) -> Result<()> {
        match extdecl {
            ExternalDeclaration::Declaration(decl) => {
                self.parse_declaration(decl)?;
            }
            ExternalDeclaration::FunctionDefinition(def) => {
                self.parse_function_definition(def)?;
            }
            // Just forward the others
            other => self.external_declarations.push(other),
        }

        Ok(())
    }

    pub fn transform_call(
        &mut self,
        fun: &mut Identifier,
        args: &mut Vec<Expr>,
        symbol_table: &HashMap<String, super::instantiate::DeclaredSymbol>,
    ) -> Result<()> {
        if let Some(template) = self.declared_templates.get(&fun.0) {
            // We found a template whose name matches the identifier
            // Thus, transform the function call

            // Extract arguments
            let mut template_parameters = template.extract_template_parameters(args)?;

            // Generate name
            let template_name = template.generate_id(&template_parameters);

            // Extract the set of captured variables
            struct Capturer<'ds> {
                st: &'ds HashMap<String, super::instantiate::DeclaredSymbol>,
                captured: HashMap<String, &'ds super::instantiate::DeclaredSymbol>,
            }

            impl Visitor for Capturer<'_> {
                fn visit_expr(&mut self, e: &mut Expr) -> Visit {
                    match e {
                        Expr::Variable(ident) => {
                            // This is a variable. If it's in the symbol table, it needs to be
                            // captured and renamed

                            if let Some(sb) = self.st.get(ident.0.as_str()) {
                                // Add it to the list of captured variables
                                self.captured.insert(ident.0.clone(), sb);
                                // Rename the reference
                                *ident = sb.gen_id.clone();
                            }
                        }
                        _ => {}
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
            let mut extra_parameters: Vec<_> = capturer.captured.into_iter().collect();
            extra_parameters.sort_by_key(|ep| ep.1.symbol_id);

            // Instantiate the template if needed
            if !self.instantiated_templates.contains(&template_name) {
                // Instantiate the template and add it to the declarations before us
                self.external_declarations
                    .push(ExternalDeclaration::FunctionDefinition(
                        template.instantiate(
                            &template_name,
                            &template_parameters,
                            &self.known_functions,
                            &extra_parameters,
                        ),
                    ));

                // Take note we instantiated the template
                self.instantiated_templates.insert(template_name.clone());
            }

            // The identifier should be replaced by the mangled name
            fun.0 = template_name;

            // Add the captured parameters to the end of the call
            for ep in extra_parameters.into_iter() {
                args.push(Expr::Variable(Identifier::new(ep.0).unwrap()));
            }
        }

        Ok(())
    }

    pub fn push_function_declaration(&mut self, def: FunctionDefinition) {
        // We discovered a new function
        self.known_functions.insert(def.prototype.name.0.clone());

        // Add the definition to the declarations
        self.external_declarations
            .push(ExternalDeclaration::FunctionDefinition(def));
    }

    pub fn into_translation_unit(self) -> Result<TranslationUnit> {
        Ok(TranslationUnit(
            NonEmpty::from_non_empty_iter(self.external_declarations.into_iter())
                .ok_or_else(|| Error::EmptyInput)?,
        ))
    }
}
