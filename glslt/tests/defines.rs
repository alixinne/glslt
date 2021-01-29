//! Tests for defines

mod common;

#[test]
fn dependent_defines() {
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
