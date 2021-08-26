//! Parsing utilities

use std::path::{Path, PathBuf};

use lang_util::located::LocatedBuilder;

use glsl_lang::{
    ast,
    parse::{ParseContext, ParseContextData, ParseOptions},
};

use glsl_lang_pp::{
    ext_name,
    exts::Registry,
    processor::{fs::FileSystem, nodes::ExtensionBehavior, ProcessorStateBuilder},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct GlsltPolicy;

impl glsl_lang::parse::TypeTablePolicy for GlsltPolicy {
    fn promote_to_type_name(
        &self,
        name: &glsl_lang::ast::Identifier,
        ctx: glsl_lang::parse::IdentifierContext,
    ) -> bool {
        trace!("glslt parsing: promoting {} to type name", name);
        ctx == glsl_lang::parse::IdentifierContext::FunctionPrototype
    }
}

fn make_parse_context(existing: Option<&ParseContext>) -> ParseContext {
    if let Some(existing) = existing {
        existing.with_policy(GlsltPolicy)
    } else {
        ParseOptions::new().with_context(ParseContextData::with_comments_and_policy(GlsltPolicy))
    }
}

/// Return a new parse builder
pub fn builder<'c>() -> ParseBuilder<'c> {
    ParseBuilder::default()
}

/// Parse operation builder
#[derive(Default, Clone, Copy)]
pub struct ParseBuilder<'c> {
    ctx: Option<&'c ParseContext>,
}

impl<'c> ParseBuilder<'c> {
    /// Create a new [ParseBuilder]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [ParseBuilder] with a given context
    pub fn with_context(ctx: impl Into<Option<&'c ParseContext>>) -> Self {
        Self { ctx: ctx.into() }
    }

    /// Create a new [ParseBuilder] with a given source
    pub fn with_source<'i>(source: &'i str) -> SourceParseBuilder<'c, 'i> {
        Self::new().source(source)
    }

    /// Create a new [ParseBuilder] with a given source
    pub fn with_source_and_context<'i>(
        source: &'i str,
        context: impl Into<Option<&'c ParseContext>>,
    ) -> SourceParseBuilder<'c, 'i> {
        Self::with_context(context).source(source)
    }

    /// Set the context for this parse operation
    pub fn context(mut self, ctx: &'c ParseContext) -> Self {
        self.ctx = Some(ctx);
        self
    }

    /// Set the source to parse in this operation
    pub fn source<'i>(self, source: &'i str) -> SourceParseBuilder<'c, 'i> {
        SourceParseBuilder {
            ctx: self.ctx,
            source,
        }
    }

    /// Set the filesystem to use for this operation
    pub fn filesystem<F: FileSystem>(self, f: F) -> FileSystemParseBuilder<'c, F> {
        FileSystemParseBuilder {
            ctx: self.ctx,
            f,
            system_paths: Default::default(),
        }
    }
}

/// Parse operation builder with a source
pub struct SourceParseBuilder<'c, 'i> {
    ctx: Option<&'c ParseContext>,
    source: &'i str,
}

impl<'c, 'i> SourceParseBuilder<'c, 'i> {
    /// Run the parse operation
    pub fn run(self) -> glsl_lang::parse::ParseResult<&'i str, ast::TranslationUnit> {
        use glsl_lang::parse::IntoLexerExt;

        let parse_context = make_parse_context(self.ctx);
        self.source.builder().opts(&parse_context).parse()
    }
}

/// Parse operation builder with a filesystem
pub struct FileSystemParseBuilder<'c, F: FileSystem> {
    ctx: Option<&'c ParseContext>,
    f: F,
    system_paths: Vec<PathBuf>,
}

impl<'c, F: FileSystem> FileSystemParseBuilder<'c, F> {
    /// Set the system paths for this operation
    pub fn system_paths(mut self, paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        self.system_paths.clear();
        self.system_paths
            .extend(paths.into_iter().map(|path| path.as_ref().to_owned()));
        self
    }

    /// Set the input source for this operation
    pub fn source<'i>(
        self,
        source: &'i str,
        path: impl AsRef<Path>,
    ) -> FileSystemParseBuilderWithSource<'c, 'i, F> {
        FileSystemParseBuilderWithSource {
            ctx: self.ctx,
            f: self.f,
            system_paths: self.system_paths,
            source,
            path: path.as_ref().to_owned(),
            registry: None,
        }
    }

    /// Set the input paths for this operation
    pub fn input(self, path: impl AsRef<Path>) -> FileSystemParseBuilderWithFile<'c, F> {
        self.inputs(Some(path))
    }

    /// Set the input paths for this operation
    pub fn inputs(
        self,
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> FileSystemParseBuilderWithFile<'c, F> {
        FileSystemParseBuilderWithFile {
            ctx: self.ctx,
            f: self.f,
            system_paths: self.system_paths,
            files: paths
                .into_iter()
                .map(|path| path.as_ref().to_owned())
                .collect(),
            registry: None,
        }
    }
}

/// Parse operation builder with a filesystem
pub struct FileSystemParseBuilderWithFile<'c, F: FileSystem> {
    ctx: Option<&'c ParseContext>,
    f: F,
    system_paths: Vec<PathBuf>,
    files: Vec<PathBuf>,
    registry: Option<&'c Registry>,
}

fn process_file<'r, 'p, F: FileSystem>(
    f: glsl_lang::lexer::v2::fs::File<'r, 'p, F>,
    ctx: &ParseContext,
) -> glsl_lang::parse::ParseResult<glsl_lang::lexer::v2::fs::File<'r, 'p, F>, ast::TranslationUnit>
{
    use glsl_lang::parse::IntoLexerExt;

    f.with_state(
        glsl_lang_pp::processor::ProcessorState::builder().extension(
            ext_name!("GL_GOOGLE_include_directive"),
            ExtensionBehavior::Enable,
        ),
    )
    .builder::<'_, '_, ast::TranslationUnit>()
    .opts(ctx)
    .parse()
}

impl<'c, F: FileSystem> FileSystemParseBuilderWithFile<'c, F> {
    /// Set the extension registry
    pub fn registry(mut self, registry: impl Into<&'c Registry>) -> Self {
        self.registry = Some(registry.into());
        self
    }

    /// Set the input path for this operation
    pub fn run(
        mut self,
    ) -> Result<
        (ast::TranslationUnit, ParseContext),
        lang_util::error::ParseError<glsl_lang::lexer::v2::LexicalError<F::Error>>,
    > {
        use glsl_lang::lexer::v2::fs::PreprocessorExt;

        // Move registry out to prevent reborrow
        let registry = self.registry.take();

        let mut processor = glsl_lang_pp::processor::fs::Processor::new_with_fs(self.f);
        processor
            .system_paths_mut()
            .extend(self.system_paths.into_iter());

        let mut external_declarations = Vec::new();

        let mut ctx = make_parse_context(self.ctx);

        for path in &self.files {
            let (tu, new_ctx, _) = process_file(
                processor
                    .open(path)
                    .map(|file| {
                        if let Some(registry) = registry {
                            file.with_registry(registry)
                                .with_state(ProcessorStateBuilder::new(registry).finish())
                        } else {
                            file
                        }
                    })
                    .map_err(|err| {
                        LocatedBuilder::new().path(path).finish(
                            lang_util::error::ParseErrorKind::LexicalError {
                                error: glsl_lang::lexer::v2::LexicalError::Io(
                                    LocatedBuilder::new().path(path).finish(err),
                                ),
                            },
                        )
                    })?,
                &ctx,
            )?;

            ctx = new_ctx;
            external_declarations.extend(tu.0);
        }

        Ok((ast::TranslationUnit(external_declarations), ctx))
    }
}

/// Parse operation builder with a filesystem
pub struct FileSystemParseBuilderWithSource<'c, 'i, F: FileSystem> {
    ctx: Option<&'c ParseContext>,
    f: F,
    system_paths: Vec<PathBuf>,
    source: &'i str,
    path: PathBuf,
    registry: Option<&'i Registry>,
}

impl<'c, 'i, F: FileSystem> FileSystemParseBuilderWithSource<'c, 'i, F> {
    /// Set the extension registry
    pub fn registry(mut self, registry: impl Into<&'i Registry>) -> Self {
        self.registry = Some(registry.into());
        self
    }

    /// Set the input path for this operation
    pub fn run(
        self,
    ) -> Result<
        (ast::TranslationUnit, ParseContext),
        lang_util::error::ParseError<glsl_lang::lexer::v2::LexicalError<F::Error>>,
    > {
        use glsl_lang::lexer::v2::fs::PreprocessorExt;

        let mut processor = glsl_lang_pp::processor::fs::Processor::new_with_fs(self.f);
        processor
            .system_paths_mut()
            .extend(self.system_paths.into_iter());

        let ctx = make_parse_context(self.ctx);
        let (tu, ctx, _) = process_file(
            {
                let file = processor.open_source(self.source, self.path);
                if let Some(registry) = self.registry {
                    file.with_registry(registry)
                        .with_state(ProcessorStateBuilder::new(registry).finish())
                } else {
                    file
                }
            },
            &ctx,
        )?;
        Ok((tu, ctx))
    }
}

/// Parse the given source with the default options and no filesystem inclusion
#[deprecated(since = "0.6.0", note = "use the ParseBuilder interface instead")]
pub fn parse_source_default(
    source: &str,
) -> glsl_lang::parse::ParseResult<&str, ast::TranslationUnit> {
    ParseBuilder::with_source(source).run()
}

/// Parse the given source without filesystem inclusion
#[deprecated(since = "0.6.0", note = "use the ParseBuilder interface instead")]
pub fn parse_source<'i>(
    source: &'i str,
    ctx: Option<&ParseContext>,
) -> glsl_lang::parse::ParseResult<&'i str, ast::TranslationUnit> {
    ParseBuilder::with_source_and_context(source, ctx).run()
}

/// Parse a set of files into a single translation unit
///
/// # Parameters
///
/// * `pb`: list of paths to concatenate
/// * `system_paths`: list of system include paths
/// * `fs`: fs implementation
/// * `ctx`: parse options
#[deprecated(since = "0.6.0", note = "use the ParseBuilder interface instead")]
pub fn parse_files<F: glsl_lang_pp::processor::fs::FileSystem>(
    pb: &[PathBuf],
    system_paths: &[impl AsRef<Path>],
    fs: F,
    ctx: Option<&ParseContext>,
) -> Result<
    (ast::TranslationUnit, ParseContext),
    lang_util::error::ParseError<glsl_lang::lexer::v2::LexicalError<F::Error>>,
> {
    ParseBuilder::with_context(ctx)
        .filesystem(fs)
        .system_paths(system_paths)
        .inputs(pb)
        .run()
}
