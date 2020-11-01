//! Basic sanity tests

mod common;

#[test]
fn basic_static_args() {
    let src = r#"int intfn();

int fnReturnsOne() { return 1; }

int fnReturnsTwo() { return 2; }

int fnTemplate(in intfn callback) { return callback(); }

void main() {
    gl_FragColor = vec4(fnTemplate(fnReturnsOne), fnTemplate(fnReturnsTwo), 0., 1.);
}"#;

    common::verify_transform(
        src,
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

    common::verify_min_transform(
        src,
        r#"
int fnReturnsOne() {
    return 1;
}

int _glslt_fnTemplate_dd5173() {
    return fnReturnsOne();
}

int fnReturnsTwo() {
    return 2;
}

int _glslt_fnTemplate_4314fd() {
    return fnReturnsTwo();
}

void main() {
    gl_FragColor = vec4(_glslt_fnTemplate_dd5173(), _glslt_fnTemplate_4314fd(), 0., 1.);
}
    "#,
        "main",
    );
}

#[test]
fn basic_lambdas() {
    let src = r#"int intfn();

int fnTemplate(in intfn callback) { return callback(); }

void main() {
    gl_FragColor = vec4(fnTemplate(1), fnTemplate(2), 0., 1.);
}"#;

    common::verify_both(
        src,
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
        "main",
    );
}
