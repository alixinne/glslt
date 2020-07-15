//! glsltcc is the default compiler front-end for the GLSLT language. To use the GLSLT transforms
//! from Rust or Python code, see the documentation for the
//! [glslt](https://vtavernier.github.io/glslt/doc/glslt/) crate.
//!
//! # Installation
//!
//! Extract the pre-built binary for your platform from the
//! [releases](https://github.com/vtavernier/glslt/releases/) somewhere in your `$PATH`.
//!
//! Alternatively, you may compile GLSLT from source and install it using `cargo install --force .`
//!
//! # Command-line usage
//!
//! ```bash
//! glsltcc 0.4.0
//! Vincent Tavernier <vince.tavernier@gmail.com>
//! GLSL Template compiler
//!
//! USAGE:
//!     glsltcc [OPTIONS] [--] [input]...
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -I <include>...                 System include paths
//!     -K, --keep-fns <keep-fns>...    List of symbols to keep for minifying mode
//!     -o, --output <output>           Output file (defaults to stdout)
//!
//! ARGS:
//!     <input>...    Input template files
//! ```
//!
//! # Examples
//!
//! ```bash
//! # Transform the GLSLT code in `sdf.glsl` to the output file `output.glsl`
//! glsltcc -o output.glsl sdf.glsl
//!
//! # Transform the GLSLT code in `sdf.glsl` to the output file `output.glsl`. Only keep
//! # transitive dependencies of the `mainImage` function (minifying mode).
//! glsltcc -o output.glsl -K mainImage sdf.glsl
//! ```

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

    /// List of symbols to keep for minifying mode
    #[structopt(short = "K", long)]
    keep_fns: Vec<String>,
}

#[paw::main]
fn main(opts: Opts) -> anyhow::Result<()> {
    // Parse input files in parallel
    let tu = glslt::parse_files(&opts.input, &opts.include)?;

    // Process the input
    let processed_input = if opts.keep_fns.is_empty() {
        glslt::transform(std::iter::once(&tu))?
    } else {
        glslt::transform_min(
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
