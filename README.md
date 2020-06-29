# glslt

[![Build Status](https://travis-ci.com/vtavernier/glslt.svg?branch=master)](https://travis-ci.com/vtavernier/glslt) [![Build status](https://ci.appveyor.com/api/projects/status/uimwa630f4v8t096/branch/master?svg=true)](https://ci.appveyor.com/project/vtavernier/glslt/branch/master) [![GitHub release](https://img.shields.io/github/v/release/vtavernier/glslt)](https://github.com/vtavernier/glslt/releases) [![License](https://img.shields.io/github/license/vtavernier/glslt)](LICENSE)

`glslt` is a prototype language for adding template functions to the GLSL
language.

Although they are not strictly needed for basic shading operations, they are of
particular interest for designing reusable GLSL components which agree on
common interfaces, as function pointers (or callbacks) would provide.

## Installation

Check out the [releases](https://github.com/vtavernier/glslt/releases) for
pre-compiled binaries for stable versions.

Alternatively, you may compile `glslt` from source, assuming you have the
[Rust](https://rustup.rs/) compiler installed:

```bash
# Fetch the source
git clone https://github.com/vtavernier/glslt.git
cd glslt

# Run the program directly
cargo run -- test.glsl

# Or, install the glsltcc binary permanently
cargo install --release --force .
glsltcc test.glsl
```

## Usage

### Static template function parameters

`glslt` supports *static template function parameters*. This means, passing the
name of an already-declared function as a parameter for a templated function.
Here is an example:

```glsl
// A pointer to a function that has no args and returns an int
//
// We use function prototypes for this matter since they're
// basically useless in GLSL. Since there can be no indirect
// recursion, there is no need for function pre-declarations.
int intfn();

// A first function that could be an intfn
int fnReturnsOne() { return 1; }

// A second function that could be an intfn
int fnReturnsTwo() { return 2; }

// A template function. It's recognized as a template because it uses intfn
// which has been declared as a function pointer.
//
// In the generated code, there will be no function called fnTemplate, as all
// calls to fnTemplate will be replaced with template specializations.
//
// Thus, callback can only be an identifier of an existing function, which
// should (later: must with type-checking) match the pointer type
int fnTemplate(in intfn callback) { return callback(); }

void main() {
    // Calling fnTemplate with function pointers
    gl_FragColor =
        vec4(fnTemplate(fnReturnsOne), fnTemplate(fnReturnsTwo), 0., 1.);
}
```

Note that we do not define a new syntax. Instead, we use the function
pre-declaration syntax which is rarely used to declare function pointers. Thus,
all your existing tooling still works with `glslt`.

In order to run this code on your GPU, you need to process it so function
templates are *instantiated* with their actual template parameters. This is
where this tool comes in:

```bash
# Assuming you installed the pre-built glsltcc binary, if running from source use `cargo run --` instead.
#
# test.glsl is our input example, output.glsl is the generated code.
glsltcc -o output.glsl test.glsl
```

The resulting code will look like this:

```glsl
int fnReturnsOne() {
    return 1;
}

int fnReturnsTwo() {
    return 2;
}

int __fnTemplate_dd5173() {
    return fnReturnsOne();
}

int __fnTemplate_4314fd() {
    return fnReturnsTwo();
}

void main() {
    gl_FragColor = vec4(__fnTemplate_dd5173(), __fnTemplate_4314fd(), 0., 1.);
}
```

Note how the template function calls have been replaced by regular GLSL
functions. This code can be directly used in an OpenGL application.

### Non-capturing lambda template function parameters

`glslt` also supports *non-capturing lambda template function parameters*.
Instead of passing a function name as a parameter to the templated function,
you may pass an expression. Currently, this expression has to be non-capturing,
meaning it cannot use local variables or parameters from the calling functions.
Here is an example:

```glsl
float sdf3d(in vec3 p);

float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float opElongate(in sdf3d primitive, in vec3 p, in vec3 h) {
    vec3 q = p - clamp(p, -h, h);
    return primitive(q);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    fragColor = vec4(vec3(opElongate(sdSphere(_1, 4.), vec3(fragCoord, 0.), vec3(1., 2., 3.))), 1.0);
}
```

Note how instead of just passing `sdSphere` as a template parameter, we pass
`sdSphere(_1, 4.)`. This translates to calling `sdSphere` with the first
parameter given by the template function `opElongate`, while the second
parameter is the constant `4.`. This results in the following code:

```glsl
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float __opElongate_d20939(in vec3 p, in vec3 h) {
    vec3 q = p - clamp(p, -h, h);
    return sdSphere(q, 4.);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    fragColor = vec4(vec3(__opElongate_d20939(vec3(fragCoord, 0.), vec3(1., 2., 3.))), 1.);
}
```

### Capturing lambda template function parameters

Capturing local variables and parameters from the calling function in the
lambda expressions is not yet supported. Implementing this feature is currently
blocked by https://github.com/phaazon/glsl/issues/72.

### Support for include directives

`#include` directives are supported and will be processed, using the same rules
as C preprocessors: double-quoted paths will be looked up from the current file
being parsed, and then fallback to the system include paths. Angle-quoted paths
will be looked up from the system include paths.

**Warning**: since include directives are processed at the AST level, shaders
which rely on included files to generate valid syntax are not supported.

## Features

- [x] Include support
- [ ] Preserve comments in original source
- [ ] Capturing lambda template function parameters
- [x] Non-capturing lambda template function parameters
- [x] Static template function parameters

## Limitations

This program is based on the excellent [glsl](https://github.com/phaazon/glsl)
crate for parsing and manipulating the GLSL AST in Rust. However, since it's
only an AST and not a full parse tree, we have currently no way of preserving
comments or original formatting.

Furthermore, since pre-processor directives have to be passed through to the
GPU for accurate execution, shaders which are syntactically invalid without
pre-processing are not supported.

## Author

Vincent Tavernier <vince.tavernier@gmail.com>
