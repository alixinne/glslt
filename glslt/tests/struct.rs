//! Tests for deep dependency in struct field types

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
