//! Command-line interface to glslt

use std::path::PathBuf;

pub use anyhow;
pub use structopt::StructOpt;

/// Command-line arguments structure
#[derive(StructOpt)]
#[structopt(name = "glsltcc", about = "GLSL Template compiler", author)]
pub struct Opts {
    /// Input template files
    input: Vec<PathBuf>,

    /// Quiet mode
    #[structopt(short, long)]
    quiet: bool,

    /// Verbose mode. Repeat to increase verbosity
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u32,

    /// Output file (defaults to stdout)
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// System include paths
    #[structopt(short = "I")]
    include: Vec<PathBuf>,

    /// List of symbols to keep for minifying mode
    #[structopt(short = "K", long)]
    keep_fns: Vec<String>,
}

/// Entry point for the glsltcc front-end
///
/// # Parameters
///
/// * `opts`: command-line options
pub fn main(opts: Opts) -> anyhow::Result<()> {
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

    // Parse input files in parallel
    let (tu, _) = crate::parse::parse_files(
        &opts.input,
        &crate::parse::StdPreprocessorFs::with_include_path(&opts.include),
        None,
    )?;

    // Process the input
    let processed_input = if opts.keep_fns.is_empty() {
        crate::transform(std::iter::once(&tu))?
    } else {
        crate::transform_min(
            std::iter::once(&tu),
            opts.keep_fns.iter().map(|it| it.as_str()),
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
