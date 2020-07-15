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
    // Parse input files in parallel
    let tu = crate::parse_files(&opts.input, &opts.include)?;

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
    glsl::transpiler::glsl::show_translation_unit(
        &mut s,
        &processed_input,
        glsl::transpiler::glsl::FormattingState::default(),
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
