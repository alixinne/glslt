use std::path::PathBuf;

use structopt::StructOpt;

use glslt::glsl;

#[derive(StructOpt)]
#[structopt(name = "glsltcc", about = "GLSL Template compiler", author)]
struct Opts {
    /// Input template files
    input: Vec<PathBuf>,

    /// Output file (defaults to stdout)
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// System include paths
    #[structopt(short = "I")]
    include: Vec<PathBuf>,
}

#[paw::main]
fn main(opts: Opts) -> anyhow::Result<()> {
    // Parse input files in parallel
    let tu = glslt::parse_files(&opts.input, &opts.include)?;

    // Process the input
    let processed_input = glslt::transform(std::iter::once(&tu))?;

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
