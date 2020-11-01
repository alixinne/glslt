//! Tests for templates with multiple function arguments

mod common;

#[test]
fn multiple_static_args() {
    common::verify_both(
        r#"int intfn();

int fnReturnsOne() { return 1; }

int fnReturnsTwo() { return 2; }

int fnTemplate(intfn cb1, intfn cb2) { return cb1() + cb2(); }

void main() {
    fnTemplate(fnReturnsOne, fnReturnsTwo);
}"#,
        r#"int fnReturnsOne() { return 1; }

int fnReturnsTwo() { return 2; }

int _glslt_fnTemplate_0() { return fnReturnsOne() + fnReturnsTwo(); }

void main() {
    _glslt_fnTemplate_0();
}"#,
        "main",
    );
}

#[test]
fn multiple_lambdas() {
    common::verify_both(
        r#"int intfn();

int fnTemplate(intfn cb1, intfn cb2) { return cb1() + cb2(); }

void main() {
    fnTemplate(1, 2);
}"#,
        r#"int _glslt_fnTemplate_0() { return 1 + 2; }

void main() {
    _glslt_fnTemplate_0();
}"#,
        "main",
    );
}
