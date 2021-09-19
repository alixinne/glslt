//! Tests for deep dependency in struct field types and blocks

mod common;

#[test]
fn nested_struct_types() {
    common::verify_min_transform(
        r#"struct A {
    float x;
};

struct B {
    A a;
};

void main() {
    B b;
}"#,
        r#"struct A {
    float x;
};

struct B {
    A a;
};

void main() {
    B b;
}"#,
        "main",
    );
}

#[test]
fn uniform_blocks() {
    // Blocks should be kept since their existence has side effects (i.e. they affect program
    // linking)

    let src = r#"uniform Block {
    float x;
    float y;
};

void main() {
    gl_FragColor = x;
}"#;

    common::verify_min_transform(src, src, "main");
}
