//! Tests for templates with multiple function arguments

mod common;

#[test]
fn nested_static_args() {
    common::verify_both(
        r#"int IntCallback();

int callbackTarget() { return 1; }

int innerTemplate(IntCallback cbi) {
    return cbi();
}

int outerTemplate(IntCallback cbo) {
    return innerTemplate(cbo);
}

void main() {
    outerTemplate(callbackTarget);
}"#,
        r#"int callbackTarget() { return 1; }

int _glslt_innerTemplate_1() {
    return callbackTarget();
}

int _glslt_outerTemplate_0() {
    return _glslt_innerTemplate_1();
}

void main() {
    _glslt_outerTemplate_0();
}"#,
        "main",
    );
}

#[test]
fn nested_static_args_with_arg() {
    common::verify_both(
        r#"int IntCallback(int x);

int callbackTarget(int x) { return x; }

int innerTemplate(IntCallback cbi) {
    return cbi(1);
}

int outerTemplate(IntCallback cbo) {
    return innerTemplate(cbo);
}

void main() {
    outerTemplate(callbackTarget);
}"#,
        r#"int callbackTarget(int x) { return x; }

int _glslt_innerTemplate_1() {
    return callbackTarget(1);
}

int _glslt_outerTemplate_0() {
    return _glslt_innerTemplate_1();
}

void main() {
    _glslt_outerTemplate_0();
}"#,
        "main",
    );
}

#[test]
fn nested_lambda_with_arg() {
    common::verify_both(
        r#"int IntCallback(int x);

int innerTemplate(IntCallback cbi) {
    return cbi(1);
}

int outerTemplate(IntCallback cbo) {
    return innerTemplate(cbo);
}

void main() {
    outerTemplate(_1);
}"#,
        r#"int _glslt_innerTemplate_1() {
    return 1;
}

int _glslt_outerTemplate_0() {
    return _glslt_innerTemplate_1();
}

void main() {
    _glslt_outerTemplate_0();
}"#,
        "main",
    );
}

#[test]
fn doubly_nested_lambda_with_arg() {
    common::verify_both(
        r#"int IntCallback(int x);

int innerTemplate(IntCallback cbi) {
    return cbi(1);
}

int outerTemplate(IntCallback cbo) {
    return innerTemplate(2 * cbo(_1));
}

void main() {
    outerTemplate(_1);
}"#,
        r#"int _glslt_innerTemplate_1() {
    return 2 * 1;
}

int _glslt_outerTemplate_0() {
    return _glslt_innerTemplate_1();
}

void main() {
    _glslt_outerTemplate_0();
}"#,
        "main",
    );
}

#[test]
fn nested_complex() {
    common::verify_both(
        r#"vec4 PathInfill(float p);

vec4 pathFillSegment(float ph, float width, PathInfill infill) {
  return infill(ph) * vec4(1., 1., 1., 0.5);
}

vec4 infillSolidBorder(float p, float width, PathInfill color) {
  return pathFillSegment(p, width, color(p));
}

void main() {
    gl_FragColor = infillSolidBorder(0.25, 5.0, vec4(1.));
}"#,
        r#"vec4 _glslt_pathFillSegment_0(float ph, float width) {
    return vec4(1.) * vec4(1., 1., 1., 0.5);
}

vec4 _glslt_infillSolidBorder_0(float p, float width) {
    return _glslt_pathFillSegment_0(p, width);
}

void main() {
    gl_FragColor = _glslt_infillSolidBorder_0(0.25, 5.0);
}"#,
        "main",
    );
}

#[test]
fn nested_complex_lambda() {
    common::verify_both(
        r#"vec4 PathInfill(float p);

vec4 pathFillSegment(float ph, float width, PathInfill infill) {
  return infill(ph) * vec4(1., 1., 1., 0.5);
}

vec4 infillSolidBorder(float p, float width, PathInfill color) {
  return pathFillSegment(p, width, color(p));
}

void main() {
    gl_FragColor = infillSolidBorder(0.25, 5.0, vec4(_1));
}"#,
        r#"vec4 _glslt_pathFillSegment_0(float ph, float width, float _glslt_lp0) {
    return vec4(_glslt_lp0) * vec4(1., 1., 1., 0.5);
}

vec4 _glslt_infillSolidBorder_0(float p, float width) {
    return _glslt_pathFillSegment_0(p, width, p);
}

void main() {
    gl_FragColor = _glslt_infillSolidBorder_0(0.25, 5.0);
}"#,
        "main",
    );
}

#[test]
fn nested_complex_lambda2() {
    common::verify_both(
        r#"vec4 PathInfill(float p);

vec4 pathFillSegment(float ph, float width, PathInfill infill) {
  return infill(ph) * vec4(1., 1., 1., 0.5);
}

vec4 infillSolidBorder(float p, float width, PathInfill color) {
  return pathFillSegment(p, width, color(p));
}

void main() {
    gl_FragColor = infillSolidBorder(0.25, 5.0, vec4(1., 2., 3., 4.));
}"#,
        r#"vec4 _glslt_pathFillSegment_0(float ph, float width) {
    return vec4(1., 2., 3., 4.) * vec4(1., 1., 1., 0.5);
}

vec4 _glslt_infillSolidBorder_0(float p, float width) {
    return _glslt_pathFillSegment_0(p, width);
}

void main() {
    gl_FragColor = _glslt_infillSolidBorder_0(0.25, 5.0);
}"#,
        "main",
    );
}
