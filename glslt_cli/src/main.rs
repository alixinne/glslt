use std::path::PathBuf;

use rayon::prelude::*;
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
}

struct ParsedInput<'p> {
    srcpath: &'p PathBuf,
    ast: glsl::syntax::TranslationUnit,
}

fn parse_input(pb: &PathBuf) -> anyhow::Result<ParsedInput> {
    Ok(ParsedInput {
        srcpath: pb,
        ast: glslt::parse(&std::fs::read_to_string(pb)?)?,
    })
}

#[paw::main]
fn main(opts: Opts) -> anyhow::Result<()> {
    // Parse input files in parallel
    let parsed_files: Result<Vec<_>, _> = opts.input.par_iter().map(parse_input).collect();
    let parsed_files = parsed_files?;

    // Process the input
    let processed_input = glslt::transform(parsed_files.iter().map(|pi| &pi.ast))?;

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
        write!(std::fs::File::open(path)?, "{}", s)?;
    } else {
        print!("{}", s);
    }

    Ok(())
}
