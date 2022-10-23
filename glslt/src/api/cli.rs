//! Command-line interface to glslt

use std::path::PathBuf;

use crate::TransformConfigBuilder;

// Re-export clap Parser
pub use clap::Parser;

/// Command-line arguments structure
#[derive(clap::Parser)]
#[command(name = "glsltc", about = "GLSL Template compiler", author)]
pub struct Opts {
    /// Input template files
    input: Vec<PathBuf>,

    /// Quiet mode
    #[arg(short, long)]
    quiet: bool,

    /// Verbose mode. Repeat to increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// System include paths
    #[arg(short = 'I')]
    include: Vec<PathBuf>,

    /// List of symbols to keep for minifying mode
    #[arg(short = 'K', long)]
    keep_fns: Vec<String>,

    /// Identifier prefix for generated code
    #[arg(short, long)]
    prefix: Option<String>,
}

/// Entry point for the glsltc front-end
///
/// # Parameters
///
/// * `opts`: command-line options
pub fn main(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .filter_or(
                "GLSLT_LOG",
                match opts.verbose {
                    0 => {
                        if opts.quiet {
                            "error"
                        } else {
                            "warn"
                        }
                    }
                    1 => "info",
                    2 => "debug",
                    _ => "trace",
                },
            )
            .write_style("GLSLT_LOG_STYLE"),
    )
    .format_timestamp(None)
    .try_init()
    .ok();

    // Parse input files
    let tu = super::common::parse_inputs_as_tu(&opts.include, &opts.input)?;

    // Process the input
    let config = {
        let mut builder = TransformConfigBuilder::default();
        if let Some(prefix) = &opts.prefix {
            builder.prefix(prefix.to_owned());
        }
        builder.build()
    };

    let processed_input = if opts.keep_fns.is_empty() {
        crate::transform(std::iter::once(&tu), config)?
    } else {
        crate::transform_min(
            std::iter::once(&tu),
            opts.keep_fns.iter().map(|it| it.as_str()),
            config,
        )?
    };

    // Transpile
    let mut s = String::new();
    glsl_lang::transpiler::glsl::show_translation_unit(
        &mut s,
        &processed_input,
        glsl_lang::transpiler::glsl::FormattingState::default(),
    )?;

    // Write output
    if let Some(path) = opts.output {
        use std::io::Write;
        write!(std::fs::File::create(path)?, "{}", s)?;
    } else {
        print!("{}", s);
    }

    Ok(())
}
