#!/bin/bash

set -eux

perl -pi -e "if (!\$a) { \$_ =~ s/^version = .*/version = \"$1\"/; \$a = 1 }" */Cargo.toml
cargo upgrade --workspace "glslt@=$1"
