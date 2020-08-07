#!/bin/bash
set -e

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
cargo test
grcov ./target/debug/ -s . -t lcov --llvm --branch -o ./target/debug/lcov.info
# Fix workspace paths
sed -i 's/SF:\(src\|tests\)/SF:glslt\/\1/' ./target/debug/lcov.info
genhtml -o ./target/debug/coverage/ --show-details --highlight --ignore-errors source --legend ./target/debug/lcov.info
