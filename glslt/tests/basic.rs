//! Basic sanity tests

mod common;

#[test]
fn basic_static_args() {
    common::verify_transform(
        r#"int intfn();

int fnReturnsOne() { return 1; }

int fnReturnsTwo() { return 2; }

int fnTemplate(in intfn callback) { return callback(); }

void main() {
    gl_FragColor = vec4(fnTemplate(fnReturnsOne), fnTemplate(fnReturnsTwo), 0., 1.);
}"#,
        r#"
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
    "#,
    );
}

#[test]
fn basic_lambdas() {
    common::verify_transform(
        r#"int intfn();

int fnTemplate(in intfn callback) { return callback(); }

void main() {
    gl_FragColor = vec4(fnTemplate(1), fnTemplate(2), 0., 1.);
}"#,
        r#"int _glslt_fnTemplate_dd5173() {
    return 1;
}

int _glslt_fnTemplate_4314fd() {
    return 2;
}

void main() {
    gl_FragColor = vec4(_glslt_fnTemplate_dd5173(), _glslt_fnTemplate_4314fd(), 0., 1.);
}
    "#,
    );
}
