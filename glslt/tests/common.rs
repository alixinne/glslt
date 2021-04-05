use std::collections::{HashMap, HashSet};

use glsl_lang::{
    ast::*,
    visitor::{HostMut, Visit, VisitorMut},
};

// Code from pretty_assertions
// Use contents_eq to ignore span information when comparing syntax trees

#[macro_export]
macro_rules! assert_eq {
    ($left:expr , $right:expr,) => ({
        assert_eq!($left, $right)
    });
    ($left:expr , $right:expr) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !left_val.content_eq(&right_val) {
                    panic!("assertion failed: `(left == right)`\
                          \n\
                          \n{}\
                          \n",
                           pretty_assertions::Comparison::new(left_val, right_val))
                }
            }
        }
    });
    ($left:expr , $right:expr, $($arg:tt)*) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !left_val.content_eq(&right_val) {
                    panic!("assertion failed: `(left == right)`: {}\
                          \n\
                          \n{}\
                          \n",
                           format_args!($($arg)*),
                           pretty_assertions::Comparison::new(left_val, right_val))
                }
            }
        }
    });
}

use glslt::transform::TransformUnit;

/// In-order generated identifier discovery
#[derive(Debug, Default)]
struct IdentifierDiscovery {
    identifiers: Vec<SmolStr>,
    known_identifiers: HashMap<String, usize>,
}

impl VisitorMut for IdentifierDiscovery {
    fn visit_identifier(&mut self, ident: &mut Identifier) -> Visit {
        if ident.0.starts_with(glslt::PREFIX) {
            // Extract generated part
            let generated = &ident.0[glslt::PREFIX.len() + 1..];

            if !self.known_identifiers.contains_key(generated) {
                self.identifiers.push(generated.into());
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
    seen_identifiers: HashMap<SmolStr, SmolStr>,
    missing_identifiers: HashSet<SmolStr>,
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
        target.visit_mut(&mut self);

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

impl VisitorMut for IdentifierReplacement<'_> {
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
                } else if self.discovery.identifiers[self.current_idx]
                    .starts_with(generated.splitn(2, '_').next().unwrap())
                {
                    // More identifiers left, take one and generate the replaced string
                    let repl = format!(
                        "{}_{}",
                        glslt::PREFIX,
                        self.discovery.identifiers[self.current_idx]
                    );
                    self.current_idx += 1;
                    self.seen_identifiers.insert(generated.into(), repl.into());
                    self.seen_identifiers.get(generated)
                } else {
                    None
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
    glsl_lang::transpiler::glsl::show_translation_unit(
        &mut s,
        tu,
        glsl_lang::transpiler::glsl::FormattingState::default(),
    )
    .unwrap();
    s
}

fn verify_transform_impl(
    src: &str,
    expected: &str,
    transform: impl FnOnce(TranslationUnit) -> TranslationUnit,
) {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init()
        .ok();

    // Parse source
    let (src, _) = glslt::parse::parse_source_default(src).expect("failed to parse src");

    // Parse expected result
    let (mut expected, _) =
        glslt::parse::parse_source_default(expected).expect("failed to parse expected");

    // Reformat source
    let source = to_string(&src);

    // Run transform function
    let mut transformed = transform(src);

    // Visit the transformed source to find generated identifiers
    let mut id = IdentifierDiscovery::default();
    expected.visit_mut(&mut id);

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

#[allow(dead_code)]
pub fn verify_transform(src: &str, expected: &str) {
    verify_transform_impl(src, expected, |src| {
        // Transform source
        let mut unit = glslt::transform::Unit::new();
        for decl in src.0.into_iter() {
            let err = format!("failed to transform declaration: {:?}", decl);
            unit.parse_external_declaration(decl).expect(&err);
        }

        // Get result
        unit.into_translation_unit()
            .expect("failed to obtain translation unit for result")
    });
}

#[allow(dead_code)]
pub fn verify_min_transform(src: &str, expected: &str, entry_point: &str) {
    verify_transform_impl(src, expected, |src| {
        // Transform source
        let mut unit = glslt::transform::MinUnit::new();
        for decl in src.0.into_iter() {
            let err = format!("failed to transform declaration: {:?}", decl);
            unit.parse_external_declaration(decl).expect(&err);
        }

        // Get result
        let entry_points = [entry_point];
        unit.into_translation_unit(entry_points.iter().copied())
            .expect("failed to obtain translation unit for result")
    });
}

#[allow(dead_code)]
pub fn verify_both(src: &str, expected: &str, entry_point: &str) {
    verify_transform(src, expected);
    verify_min_transform(src, expected, entry_point);
}
