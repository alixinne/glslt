mod common;

#[test]
fn propagate_reinst() {
    common::verify_both(
        r#"vec4 ColorFunction(float phase);

vec4 filler(float phase, float width, ColorFunction inner) {
    return width * inner(phase);
}

vec4 infillSolidBorder(float phase, float width, ColorFunction cfn) {
    return filler(phase, width, cfn(phase));
}

vec4 layerBody(vec4 prevColor) {
    return infillSolidBorder(12.5, 2.0, vec4(prevColor.xyz / _1, 1.0));
}

void main() {
    gl_FragColor = layerBody(vec4(0., 0., 0., 1.));
}
"#,
        r#"vec4 _glslt_filler_d599b3(float phase, float width, vec4 _glslt_lp0, float _glslt_lp1) {
    return width * vec4(_glslt_lp0.xyz / _glslt_lp1, 1.);
}

vec4 _glslt_infillSolidBorder_9fa250(float phase, float width, vec4 _glslt_lp0) {
    return _glslt_filler_d599b3(phase, width, _glslt_lp0, phase);
}

vec4 layerBody(vec4 prevColor) {
    return _glslt_infillSolidBorder_9fa250(12.5, 2., prevColor);
}

void main() {
    gl_FragColor = layerBody(vec4(0., 0., 0., 1.));
}
"#,
        "main",
    )
}

#[test]
fn propagate_static() {
    common::verify_both(
        r#"vec4 ColorFunction(float phase);

vec4 filler(float phase, float width, ColorFunction inner) {
    return width * inner(phase);
}

vec4 infillSolidBorder(float phase, float width, ColorFunction cfn) {
    return filler(phase, width, cfn);
}

vec4 layerBody(vec4 prevColor) {
    return infillSolidBorder(12.5, 2.0, vec4(prevColor.xyz / _1, 1.0));
}

void main() {
    gl_FragColor = layerBody(vec4(0., 0., 0., 1.));
}
"#,
        r#"vec4 _glslt_filler_d599b3(float phase, float width, vec4 _glslt_lp0) {
    return width * vec4(_glslt_lp0.xyz / phase, 1.);
}

vec4 _glslt_infillSolidBorder_9fa250(float phase, float width, vec4 _glslt_lp0) {
    return _glslt_filler_d599b3(phase, width, _glslt_lp0);
}

vec4 layerBody(vec4 prevColor) {
    return _glslt_infillSolidBorder_9fa250(12.5, 2., prevColor);
}

void main() {
    gl_FragColor = layerBody(vec4(0., 0., 0., 1.));
}
"#,
        "main",
    )
}
