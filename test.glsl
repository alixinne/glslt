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
