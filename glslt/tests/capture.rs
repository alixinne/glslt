//! Variable capture tests

mod common;

#[test]
fn capture_simple() {
    common::verify_both(
        r#"int intfn(int x);

int fnTemplate(in intfn callback) {
    return callback(1);
}

void main() {
    int captureMe = 8;
    fnTemplate(_1 * captureMe);
}"#,
        r#"int _glslt_fnTemplate_0(int _glslt_lp0) {
    return 1 * _glslt_lp0;
}

void main() {
    int captureMe = 8;
    _glslt_fnTemplate_0(captureMe);
}"#,
        "main",
    );
}

#[test]
fn capture_double() {
    common::verify_both(
        r#"int intfn(int x, int y);

int fnTemplate(in intfn callback) {
    int extra = 2;
    return callback(1, extra);
}

void main() {
    int captureMe = 8;
    fnTemplate(_1 * _2 * captureMe);
}"#,
        r#"int _glslt_fnTemplate_0(int _glslt_lp0) {
    int extra = 2;
    return 1 * extra * _glslt_lp0;
}

void main() {
    int captureMe = 8;
    _glslt_fnTemplate_0(captureMe);
}"#,
        "main",
    );
}

#[test]
fn capture_static() {
    common::verify_both(
        r#"int Fn1(int x);
int Fn2(int x, int y);

int target2(int x, int y) {
    return x * y;
}

int target1(int x) {
    return 2 * x;
}

int fnTemplate(Fn1 cb1, Fn2 cb2) {
    int extra = 2;
    return cb2(cb1(extra), extra);
}

void main() {
    int captureMe = 8;
    fnTemplate(target1, target2(_1, _2 * captureMe));
}"#,
        r#"int target2(int x, int y) {
    return x * y;
}

int target1(int x) {
    return 2 * x;
}

int _glslt_fnTemplate_0(int _glslt_lp0) {
    int extra = 2;
    return target2(target1(extra), extra * _glslt_lp0);
}

void main() {
    int captureMe = 8;
    _glslt_fnTemplate_0(captureMe);
}"#,
        "main",
    );
}

#[test]
fn capture_stack() {
    // This test passing ensures there is no infinite recursion in lambda_instantiate

    common::verify_both(
        r#"int Fn1(int x);
int Fn2(int x, int y);

int target2(int x, int y) {
    return x * y;
}

int fnTemplate(Fn1 cb1, Fn2 cb2) {
    int extra = 2;
    return cb2(cb1(_1 * extra), extra);
}

void main() {
    int captureMe = 8;
    fnTemplate(target2(_1, _2 * captureMe));
}"#,
        r#"int target2(int x, int y) {
        return x * y;
}

int _glslt_fnTemplate_0(int _glslt_lp0) {
        int extra = 2;
            return cb2(target2(_1 * extra, _2 * _glslt_lp0), extra);
}

void main() {
        int captureMe = 8;
            _glslt_fnTemplate_0(captureMe);
}"#,
        "main",
    );
}
