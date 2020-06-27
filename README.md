# glslt

[![Build Status](https://travis-ci.com/vtavernier/glslt.svg?branch=master)](https://travis-ci.com/vtavernier/glslt) [![Build status](https://ci.appveyor.com/api/projects/status/uimwa630f4v8t096/branch/master?svg=true)](https://ci.appveyor.com/project/vtavernier/glslt/branch/master) [![GitHub release](https://img.shields.io/github/v/release/vtavernier/glslt)](https://github.com/vtavernier/glslt/releases) [![License](https://img.shields.io/github/license/vtavernier/glslt)](LICENSE)

`glslt` is a prototype language for adding template functions to the GLSL
language.

Although they are not strictly needed for basic shading operations, they are of
particular interest for designing reusable GLSL components which agree on
common interfaces, as function pointers (or callbacks) would provide.

## Usage

Currently `glslt` supports only template function parameters: passing functions
as parameters to other functions. Here is an example:

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
# Assuming you installed the pre-built glslcc binary, if running from source use `cargo run --` instead.
#
# test.glsl is our input example, output.glsl is the generated code.
glslcc -o output.glsl test.glsl
```

The resulting code will look like this:

```glsl
int fnReturnsOne() {
    return 1;
}

int fnReturnsTwo() {
    return 2;
}

int __fnTemplate_fnReturnsOne() {
    return fnReturnsOne();
}

int __fnTemplate_fnReturnsTwo() {
    return fnReturnsTwo();
}

void main() {
    gl_FragColor = vec4(__fnTemplate_fnReturnsOne(), __fnTemplate_fnReturnsTwo(), 0., 1.);
}
```

Note how the template function calls have been replaced by regular GLSL
functions. This code can be directly used in an OpenGL application.

## Features

- [ ] Include support
- [ ] Preserve comments in original source
- [x] Template function parameters

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
