//! glsltc is the default compiler front-end for the GLSLT language. To use the GLSLT transforms
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
//! glsltc 0.5.0
//! Vincent Tavernier <vince.tavernier@gmail.com>
//! GLSL Template compiler
//!
//! USAGE:
//!     glsltc [FLAGS] [OPTIONS] [--] [input]...
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -q, --quiet      Quiet mode
//!     -V, --version    Prints version information
//!     -v, --verbose    Verbose mode. Repeat to increase verbosity
//!
//! OPTIONS:
//!     -I <include>...                 System include paths
//!     -K, --keep-fns <keep-fns>...    List of symbols to keep for minifying mode
//!     -o, --output <output>           Output file (defaults to stdout)
//!     -p, --prefix <prefix>           Identifier prefix for generated code
//!
//! ARGS:
//!     <input>...    Input template files
//! ```
//!
//! # Examples
//!
//! ```bash
//! # Transform the GLSLT code in `sdf.glsl` to the output file `output.glsl`
//! glsltc -o output.glsl sdf.glsl
//!
//! # Transform the GLSLT code in `sdf.glsl` to the output file `output.glsl`. Only keep
//! # transitive dependencies of the `mainImage` function (minifying mode).
//! glsltc -o output.glsl -K=mainImage sdf.glsl
//! ```

use glslt::api::cli::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    glslt::api::cli::main(Opts::from_args())
}
