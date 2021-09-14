//! Tests for defines

mod common;

#[test]
fn dependent_defines() {
    // Note: this test used to validate that glslt propagates defines correctly when glsl-lang
    // didn't have a proper preprocessor. Now that it does, these two source strings actually
    // parse into the same AST and as such don't require special handling. We keep this for non
    // regression though.

    let src = r#"#define M_PI 3.14
#define M_2PI (2. * M_PI)

void main() {
    gl_FragColor = vec4(M_2PI);
}"#;

    common::verify_min_transform(
        src,
        r#"#define M_PI 3.14
#define M_2PI (2. * M_PI)

void main() {
    gl_FragColor = vec4(M_2PI);
}
    "#,
        "main",
    );
}

#[test]
fn version_directive() {
    let src = r#"#version 460 core"#;
    common::verify_min_transform(src, src, "main");
}
