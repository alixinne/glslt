use std::path::Path;

use glsl_lang::{ast, lexer::v2_full::fs::PreprocessorExt, parse::IntoParseBuilderExt};

use glsl_lang_pp::{ext_name, processor::nodes::ExtensionBehavior};

pub fn parse_inputs_as_tu(
    include: impl IntoIterator<Item = impl AsRef<Path>>,
    inputs: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<ast::TranslationUnit, Box<dyn std::error::Error>> {
    let mut processor = glsl_lang_pp::processor::fs::StdProcessor::new();
    processor
        .system_paths_mut()
        .extend(include.into_iter().map(|path| path.as_ref().to_owned()));

    // Join translation units
    let mut external_decls = Vec::new();
    let mut context = crate::parse::make_parse_context(None);

    for input in inputs {
        let (mut tu, ctx, lexer): (ast::TranslationUnit, _, _) = processor
            .open(input.as_ref())?
            .with_state(
                glsl_lang_pp::processor::ProcessorState::builder().extension(
                    ext_name!("GL_GOOGLE_include_directive"),
                    ExtensionBehavior::Enable,
                ),
            )
            .builder()
            .context(&context)
            .parse()?;
        context = ctx;
        lexer.into_directives().inject(&mut tu);
        external_decls.extend(tu.0.into_iter());
    }

    // Create the root unit
    Ok(ast::TranslationUnit(external_decls))
}

#[cfg(feature = "python")]
pub fn parse_string(
    source: impl AsRef<str>,
) -> Result<ast::TranslationUnit, Box<dyn std::error::Error>> {
    let context = crate::parse::make_parse_context(None);
    let (mut tu, _, lexer) = source.as_ref().builder().context(&context).parse()?;
    lexer.into_directives().inject(&mut tu);
    Ok(tu)
}
