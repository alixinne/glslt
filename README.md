# [glslt](https://github.com/alixinne/glslt)

[![Run tests](https://github.com/alixinne/glslt/workflows/Run%20tests/badge.svg?branch=master)](https://github.com/alixinne/glslt/actions) [![GitHub release](https://img.shields.io/github/v/release/alixinne/glslt)](https://github.com/alixinne/glslt/releases) [![PyPI](https://img.shields.io/pypi/v/glslt)](https://pypi.org/project/glslt/) [![License](https://img.shields.io/github/license/alixinne/glslt)](LICENSE)

`glslt` is a prototype language for adding template functions to the GLSL
language.

Although they are not strictly needed for basic shading operations, they are of
particular interest for designing reusable GLSL components which agree on
common interfaces, as function pointers (or callbacks) would provide.

## Table of contents

<!-- vim-markdown-toc GFM -->

* [Installation](#installation)
* [Usage](#usage)
    * [Static template function parameters](#static-template-function-parameters)
    * [Lambda template function parameters](#lambda-template-function-parameters)
        * [Named placeholders](#named-placeholders)
    * [Nested lambda expressions](#nested-lambda-expressions)
    * [Support for include directives](#support-for-include-directives)
    * [Minifying mode](#minifying-mode)
* [Features](#features)
* [Bindings](#bindings)
    * [Rust](#rust)
    * [Python](#python)
* [Limitations](#limitations)
* [Author](#author)

<!-- vim-markdown-toc -->

## Installation

Check out the [releases](https://github.com/alixinne/glslt/releases) for
pre-compiled binaries for stable versions. Installing the Python module (see
below) also installs the corresponding `glsltc` binary.

Alternatively, you may compile `glslt` from source, assuming you have the
[Rust](https://rustup.rs/) compiler installed:

```bash
# From crates.io
cargo install --force glslt_cli

# From the repository
## Fetch the source
git clone https://github.com/alixinne/glslt.git
cd glslt

## Run the program directly
cargo run -- test.glsl

## Or, install the glsltc binary permanently
cargo install --force --path .
glsltc test.glsl
```

To install the Python module for the latest stable version, you can use `pip`:

```bash
pip install glslt
```

If you downloaded the source and want to build the latest version of the Python
module, use `maturin`:

```bash
# Install maturin (only needed once)
pip install maturin==0.13.6

# Assuming you are in a virtualenv
(cd glslt && maturin develop --features python -b pyo3)
```

## Usage

### Static template function parameters

`glslt` supports *static template function parameters*. This means, passing the
name of an already-declared function as a parameter for a templated function.
Here is an example:

```glsl
#version 460 core

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
# Assuming you installed the pre-built glsltc binary, if running from source use `cargo run --` instead.
#
# test.glsl is our input example, output.glsl is the generated code.
glsltc -o output.glsl test.glsl
```

The resulting code will look like this:

```glsl
#version 460 core
int fnReturnsOne() {
    return 1;
}

int fnReturnsTwo() {
    return 2;
}

int _glslt_fnTemplate_dd5173() {
    return fnReturnsOne();
}

int _glslt_fnTemplate_4314fd() {
    return fnReturnsTwo();
}

void main() {
    gl_FragColor = vec4(_glslt_fnTemplate_dd5173(), _glslt_fnTemplate_4314fd(), 0., 1.);
}
```

Note how the template function calls have been replaced by regular GLSL
functions. This code can be directly used in an OpenGL application.

### Lambda template function parameters

`glslt` also supports *lambda template function parameters*. Instead of passing
a function name as a parameter to the templated function, you may pass an
expression. This expression may capture local variables and parameters, which
will be taken into account when instantiating the template. Here is an example:

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

float _glslt_opElongate_d20939(in vec3 p, in vec3 h) {
    vec3 q = p - clamp(p, -h, h);
    return sdSphere(q, 4.);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    fragColor = vec4(vec3(_glslt_opElongate_d20939(vec3(fragCoord, 0.), vec3(1., 2., 3.))), 1.);
}
```

Since captures are supported, this example may have been written with the
sphere diameter being a parameter:

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
    float sz = 5.;
    fragColor = vec4(vec3(opElongate(sdSphere(_1, sz), vec3(fragCoord, 0.), vec3(1., 2., 3.))), 1.0);
    //                                            ^^
    // Using a local variable in the template argument
}
```

The variable is properly captured in the generated code:

```glsl
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

// _glslt_lp2 is the captured variable input
float _glslt_opElongate_d9170f(in vec3 p, in vec3 h, float _glslt_lp2) {
    vec3 q = p - clamp(p, -h, h);
    return sdSphere(q, _glslt_lp2);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    float sz = 5.;
    fragColor = vec4(vec3(_glslt_opElongate_d9170f(vec3(fragCoord, 0.), vec3(1., 2., 3.), sz)), 1.);
    //                                                                 Captured variable: ^^
}
```

#### Named placeholders

When passing a lambda expression to a template function, you may use the
unnamed placeholders `_1`, `_2`, etc. to refer to the first, second, etc.
arguments to the template function call. You may also use the parameter names
as declared in the function prototype. The previous example could be written as
follows:

```glsl
// In sdf3d template parameters, `p` is the first parameter name
float sdf3d(in vec3 p);

float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float opElongate(in sdf3d primitive, in vec3 p, in vec3 h) {
    vec3 q = p - clamp(p, -h, h);
    return primitive(q);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    fragColor = vec4(vec3(opElongate(sdSphere(_p, 1.0), vec3(fragCoord, 0.), vec3(1., 2., 3.))), 1.0);
    //                                        ^^
    // Named placeholder parameter in a template instead of _1
}
```

The generated code will look like this:

```glsl
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float _glslt_opElongate_784a47(in vec3 p, in vec3 h) {
    vec3 q = p - clamp(p, -h, h);
    return sdSphere(q, 1.);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    fragColor = vec4(vec3(_glslt_opElongate_784a47(vec3(fragCoord, 0.), vec3(1., 2., 3.))), 1.);
}
```

### Nested lambda expressions

Nested lambda expressions are supported, however due to the syntax being used,
we have to make a decision on how to resolve the anonymous placeholders to
their corresponding lambda. The current algorithm transforms innermost lambdas
first, so the placeholders will resolve to the most nested expression first.

This may be circumvented by using named placeholders (as long as there is no
conflict) since undefined identifiers are passed as-is to the other passes of
the transformation algorithm, and thus, to outer lambdas.

### Support for include directives

`#include` directives are supported and will be processed, using the same rules
as C preprocessors: double-quoted paths will be looked up from the current file
being parsed. Angle-quoted paths will be looked up from the system include
paths.

### Minifying mode

In its default mode, the GLSLT compiler will copy all input declarations to its
output (except function prototypes) and insert instantiated templates right
before they are used.

However, if you are using the GLSLT compiler with a large template library,
this will generate a lot of unused code. By using the `-K, --keep-fns` argument
to the `glsltc` command, GLSLT switches to the minifying mode. In this mode,
only the functions, types, and globals that are transitive dependencies of the
functions specified by the `-K` argument are kept.

`#version`, `#extension` and precision specifiers will be included at the top
of the generated code, if they were present in the input.

As an example, compiling the previous example with `glsltc -K=sdSphere` will
only return the code for the sdSphere function, since it has no dependencies.

## Features

- [x] Include support
- [ ] Preserve comments in original source
- [ ] Report position in compiler errors
- [x] Lambda template function parameters
- [x] Static template function parameters

## Bindings

### Rust

Since this tool is developed in Rust, the *native* bindings are exposed as the
`glslt` Rust crate and can be used directly by client code.

### Python

The main library exposes a native Python 3 module using
[pyo3](https://pyo3.rs/). Use [maturin](https://github.com/PyO3/maturin) to
build and develop the Python module, inside a virtualenv.

In order to build the Python package, you have to enable the `python` feature
and have `python3-dev` installed. See the documentation for [glslt](glslt/) for
an example.

## Limitations

This program is based on the [glsl-lang](https://github.com/alixinne/glsl-lang)
crate for parsing and manipulating the GLSL AST in Rust. However, since it's
only an AST and not a full parse tree, we have currently no way of preserving
original formatting. Comments are still parsed and are available to library
users, but they are not currently included in the output.

## Author

Alixinne <alixinne@pm.me>
