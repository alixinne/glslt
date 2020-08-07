use std::collections::{HashMap, HashSet};

use glsl::parser::Parse;
use glsl::syntax::*;
use glsl::visitor::{Host, Visit, Visitor};

use pretty_assertions::assert_eq;

use glslt::transform::TransformUnit;

/// In-order generated identifier discovery
#[derive(Debug, Default)]
struct IdentifierDiscovery {
    identifiers: Vec<String>,
    known_identifiers: HashMap<String, usize>,
}

impl Visitor for IdentifierDiscovery {
    fn visit_identifier(&mut self, ident: &mut Identifier) -> Visit {
        if ident.0.starts_with(glslt::PREFIX) {
            // Extract generated part
            let generated = &ident.0[glslt::PREFIX.len() + 1..];

            if !self.known_identifiers.contains_key(generated) {
                self.identifiers.push(generated.to_owned());
                self.known_identifiers
                    .insert(generated.to_owned(), self.identifiers.len() - 1);
            }
        }

        Visit::Children
    }
}

struct IdentifierReplacement<'d> {
    discovery: &'d IdentifierDiscovery,
    current_idx: usize,
    seen_identifiers: HashMap<String, String>,
    missing_identifiers: HashSet<String>,
}

impl<'d> IdentifierReplacement<'d> {
    pub fn new(discovery: &'d IdentifierDiscovery) -> Self {
        Self {
            discovery,
            current_idx: 0,
            seen_identifiers: HashMap::new(),
            missing_identifiers: HashSet::new(),
        }
    }

    pub fn replace(mut self, target: &mut TranslationUnit) -> Result<(), String> {
        target.visit(&mut self);

        if self.missing_identifiers.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "missing identifiers: {}",
                self.missing_identifiers
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }
}

impl Visitor for IdentifierReplacement<'_> {
    fn visit_identifier(&mut self, ident: &mut Identifier) -> Visit {
        if ident.0.starts_with(glslt::PREFIX) && ident.0.len() > glslt::PREFIX.len() {
            // Extract generated part
            let generated = &ident.0[glslt::PREFIX.len() + 1..];

            let replace_with = if let Some(repl) = self.seen_identifiers.get(generated) {
                // We already have a known replacement string for this
                Some(repl)
            } else {
                // This is a new identifier, consume one from the discovery
                if self.current_idx >= self.discovery.identifiers.len() {
                    // No more identifiers, something went wrong (i.e. syntax trees don't match)
                    None
                } else {
                    if self.discovery.identifiers[self.current_idx]
                        .starts_with(generated.splitn(2, "_").next().unwrap())
                    {
                        // More identifiers left, take one and generate the replaced string
                        let repl = format!(
                            "{}_{}",
                            glslt::PREFIX,
                            self.discovery.identifiers[self.current_idx]
                        );
                        self.current_idx += 1;
                        self.seen_identifiers.insert(generated.to_owned(), repl);
                        self.seen_identifiers.get(generated)
                    } else {
                        None
                    }
                }
            };

            if let Some(repl) = replace_with {
                ident.0 = repl.clone();
            } else {
                self.missing_identifiers.insert(ident.0.clone());
            }
        }

        Visit::Children
    }
}

fn to_string(tu: &TranslationUnit) -> String {
    let mut s = String::new();
    glsl::transpiler::glsl::show_translation_unit(
        &mut s,
        tu,
        glsl::transpiler::glsl::FormattingState::default(),
    )
    .unwrap();
    s
}

pub fn verify_transform(src: &str, expected: &str) {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init()
        .ok();

    // Parse source
    let src = TranslationUnit::parse(src).expect("failed to parse src");
    // Parse expected result
    let mut expected = TranslationUnit::parse(expected).expect("failed to parse expected");

    // Reformat source
    let source = to_string(&src);

    // Transform source
    let mut unit = glslt::transform::Unit::new();
    for decl in (src.0).0.into_iter() {
        let err = format!("failed to transform declaration: {:?}", decl);
        unit.parse_external_declaration(decl).expect(&err);
    }

    // Get result
    let mut transformed = unit
        .into_translation_unit()
        .expect("failed to obtain translation unit for result");

    // Visit the transformed source to find generated identifiers
    let mut id = IdentifierDiscovery::default();
    expected.visit(&mut id);

    // Transform identifiers in the expected result (assuming same order)
    let replacement = IdentifierReplacement::new(&id);
    match replacement.replace(&mut transformed) {
        Ok(()) => {
            eprintln!(
                ">>> source: \n{}\n>>> transformed: \n{}\n>>> expected: \n{}",
                source,
                to_string(&transformed),
                to_string(&expected)
            );

            assert_eq!(transformed, expected);
        }
        Err(err) => {
            eprintln!(
                ">>> source: \n{}\n>>> transformed: \n{}\n>>> expected: \n{}",
                source,
                to_string(&transformed),
                to_string(&expected)
            );

            panic!("failed to substitute identifiers in expected: {:?}", err);
        }
    }
}
