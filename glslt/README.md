# glslt

## Table of contents

<!-- vim-markdown-toc GFM -->

* [Usage](#usage)
    * [Rust crate](#rust-crate)
    * [Python library](#python-library)
* [Author](#author)

<!-- vim-markdown-toc -->

## Usage

### Rust crate

See the docs for the last [stable version](https://docs.rs/glslt/) or the
[master branch](https://vtavernier.github.io/glslt/doc/glslt/).

### Python library

If you installed the glslt library via `pip install glslt` or `maturin
develop`, you may use the Python interface to the GLSLT compiler.

```python
import glslt

# Parse the `sdf.glsl` file with `my-glsl-lib/include` being a system include
# directory for #include resolution
translation_unit = glslt.parse_files(["sdf.glsl"], ["my-glsl-lib/include"])

# Create a new minimizing transform unit
unit = glslt.MinUnit()

# Add the parsed declarations to the transform unit
unit.add_unit(translation_unit)

# Get the output of the transform
result = unit.to_translation_unit(["mainImage"])

# Print the GLSL code
print(result.to_glsl())
```

## Author

Vincent Tavernier <vince.tavernier@gmail.com>
