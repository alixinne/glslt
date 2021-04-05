#!/bin/bash

set -eux

. ci/vars

perl -pi -e "s/^version = .*/version = \"$1\"/" */Cargo.toml
cargo upgrade --workspace "glslt@$1"
