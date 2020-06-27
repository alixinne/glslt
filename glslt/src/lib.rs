use std::collections::{HashMap, HashSet};

use thiserror::Error;

pub use glsl;
use glsl::syntax::*;
use glsl::visitor::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no top-level declarations were found in any of the input files")]
    EmptyInput,
    #[error("function pointer {name} previously declared as {previous_declaration}")]
    DuplicatePointerDefinition {
        name: String,
        previous_declaration: String,
    },
    #[error("template {name} parameter #{index} cannot have an array specifier")]
    ArrayedTemplateParameter { name: String, index: usize },
    #[error("invalid template parameter #{index} for {name} instantiation")]
    InvalidTemplateParameter { name: String, index: usize },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse(src: &str) -> std::result::Result<TranslationUnit, glsl::parser::ParseError> {
    use glsl::parser::Parse;
    TranslationUnit::parse(src)
}

fn prototype_to_string(prototype: &FunctionPrototype) -> String {
    let mut s = String::new();
    glsl::transpiler::glsl::show_function_prototype(
        &mut s,
        prototype,
        &mut glsl::transpiler::glsl::FormattingState::default(),
    )
    .unwrap();
    s
}

struct TemplateParameter {
    /// Name of the function pointer type
    typename: String,
    /// Template variable name
    symbol: Option<String>,
    /// Original parameter index
    index: usize,
}

struct TemplateDefinition {
    /// AST for the partially instantiated template definition.
    ///
    /// This has to be cloned and visited to replace the template parameters.
    ast: FunctionDefinition,
    /// List of template parameters
    parameters: Vec<TemplateParameter>,
}

impl TemplateDefinition {
    pub fn instantiate(&self, name: &str, parameters: &Vec<String>) -> FunctionDefinition {
        // Clone the AST
        let mut ast = self.ast.clone();

        // Declare the visitor for the substitution
        struct V<'s> {
            subs: HashMap<&'s str, &'s str>,
        }

        impl Visitor for V<'_> {
            fn visit_expr(&mut self, e: &mut Expr) -> Visit {
                match e {
                    Expr::FunCall(fun, _) => {
                        // Only consider raw identifiers for function names
                        if let FunIdentifier::Identifier(ident) = fun {
                            if let Some(res) = self.subs.get(ident.0.as_str()) {
                                ident.0 = res.to_string();
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
                subs.insert(ps.as_str(), value.as_str());
            }
        }

        ast.visit(&mut V { subs });

        // Change the name
        ast.prototype.name.0 = name.to_string();

        ast
    }

    pub fn extract_template_parameters(&self, args: &mut Vec<Expr>) -> Result<Vec<String>> {
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
        res.into_iter()
            .enumerate()
            .map(|(id, r)| match r {
                Expr::Variable(ident) => Ok(ident.0.clone()),
                _ => Err(Error::InvalidTemplateParameter {
                    index: self.parameters[id].index,
                    name: self.ast.prototype.name.0.clone(),
                }),
            })
            .collect()
    }
}

enum TryTemplate {
    Template(TemplateDefinition),
    Function(FunctionDefinition),
}

fn parse_definition_as_template(
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

struct InstantiateTemplate<'c> {
    declared_templates: &'c mut HashMap<String, TemplateDefinition>,
    instantiated_templates: &'c mut HashSet<String>,
    external_declarations: &'c mut Vec<ExternalDeclaration>,
    error: Option<Error>,
}

impl InstantiateTemplate<'_> {
    pub fn instantiate(&mut self, mut def: FunctionDefinition) -> Result<()> {
        // Transform definition. The visitor is responsible for instantiating templates
        def.visit(self);

        if let Some(error) = self.error.take() {
            return Err(error);
        }

        // Add the definition to the declarations
        self.external_declarations
            .push(ExternalDeclaration::FunctionDefinition(def));

        Ok(())
    }

    fn transform_call(&mut self, fun: &mut Identifier, args: &mut Vec<Expr>) -> Result<()> {
        if let Some(template) = self.declared_templates.get(&fun.0) {
            // We found a template whose name matches the identifier
            // Thus, transform the function call

            // Extract arguments
            let template_parameters = template.extract_template_parameters(args)?;

            // Generate name
            let mut template_name = vec!["_", template.ast.prototype.name.0.as_str()];
            template_name.extend(template_parameters.iter().map(|s| s.as_str()));
            let template_name = template_name[..].join("_");

            // Instantiate the template if needed
            if !self.instantiated_templates.contains(&template_name) {
                // Instantiate the template and add it to the declarations before us
                self.external_declarations
                    .push(ExternalDeclaration::FunctionDefinition(
                        template.instantiate(&template_name, &template_parameters),
                    ));

                // Take note we instantiated the template
                self.instantiated_templates.insert(template_name.clone());
            }

            // The identifier should be replaced by the mangled name
            fun.0 = template_name;
        }

        Ok(())
    }
}

impl Visitor for InstantiateTemplate<'_> {
    fn visit_expr(&mut self, e: &mut Expr) -> Visit {
        match e {
            Expr::FunCall(fun, args) => {
                // Only consider raw identifiers for function names
                if let FunIdentifier::Identifier(ident) = fun {
                    if let Err(error) = self.transform_call(ident, args) {
                        self.error = Some(error);
                    }
                }
            }
            _ => {}
        }

        Visit::Children
    }
}

pub fn transform<'a>(
    asts: impl std::iter::Iterator<Item = &'a TranslationUnit>,
) -> Result<TranslationUnit> {
    let mut external_declarations = Vec::new();
    let mut declared_pointer_types = HashMap::new();
    let mut declared_templates = HashMap::new();
    let mut instantiated_templates = HashSet::new();

    for (_id, ast) in asts.enumerate() {
        // We clone all declarations since they all have somewhere to go
        for extdecl in (ast.0).0.iter().cloned() {
            match extdecl {
                ExternalDeclaration::Declaration(decl) => {
                    match decl {
                        Declaration::FunctionPrototype(prototype) => {
                            // A function prototype is what we'll call a function pointer type

                            if let Some(previous) = declared_pointer_types.get(&prototype.name.0) {
                                // Since function pointer types are raw identifiers, they have to
                                // be unique
                                return Err(Error::DuplicatePointerDefinition {
                                    name: prototype.name.0.clone(),
                                    previous_declaration: prototype_to_string(previous),
                                });
                            } else {
                                declared_pointer_types.insert(prototype.name.0.clone(), prototype);
                            }
                        }
                        other => {
                            external_declarations.push(ExternalDeclaration::Declaration(other))
                        }
                    }
                }
                ExternalDeclaration::FunctionDefinition(def) => {
                    // A function definition is a template if any of its arguments is a pointer
                    let name = def.prototype.name.0.clone();
                    let template = parse_definition_as_template(def, &declared_pointer_types)?;

                    match template {
                        TryTemplate::Template(template) => {
                            // We found a template parameter, so it's a template function
                            declared_templates.insert(name, template);
                        }
                        TryTemplate::Function(def) => {
                            // No template parameter, it's a "regular" function so it has to be
                            // processed to instantiate parameters
                            InstantiateTemplate {
                                instantiated_templates: &mut instantiated_templates,
                                external_declarations: &mut external_declarations,
                                declared_templates: &mut declared_templates,
                                error: None,
                            }
                            .instantiate(def)?;
                        }
                    }
                }
                // Just forward the others
                other => external_declarations.push(other),
            }
        }
    }

    Ok(TranslationUnit(
        NonEmpty::from_non_empty_iter(external_declarations.into_iter())
            .ok_or_else(|| Error::EmptyInput)?,
    ))
}
