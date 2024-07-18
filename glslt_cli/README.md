# glslt_cli

glsltc is the default compiler front-end for the GLSLT language. To use the GLSLT transforms
from Rust or Python code, see the documentation for the
[glslt](https://alixinne.github.io/glslt/doc/glslt/) crate.

## Installation

Extract the pre-built binary for your platform from the
[releases](https://github.com/alixinne/glslt/releases/) somewhere in your `$PATH`.

Alternatively, you may compile GLSLT from source and install it using `cargo install --force .`

## Command-line usage

```bash
GLSL Template compiler

Usage: glsltc [OPTIONS] [INPUT]...

Arguments:
  [INPUT]...  Input template files

Options:
  -q, --quiet                Quiet mode
  -v, --verbose...           Verbose mode. Repeat to increase verbosity
  -o, --output <OUTPUT>      Output file (defaults to stdout)
  -I <INCLUDE>               System include paths
  -K, --keep-fns <KEEP_FNS>  List of symbols to keep for minifying mode
  -p, --prefix <PREFIX>      Identifier prefix for generated code
  -h, --help                 Print help information
```

## Examples

```bash
# Transform the GLSLT code in `sdf.glsl` to the output file `output.glsl`
glsltc -o output.glsl sdf.glsl

# Transform the GLSLT code in `sdf.glsl` to the output file `output.glsl`. Only keep
# transitive dependencies of the `mainImage` function (minifying mode).
glsltc -o output.glsl -K=mainImage sdf.glsl
```

## Author

Alixinne <alixinne@pm.me>
