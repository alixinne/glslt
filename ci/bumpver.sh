#!/bin/bash

set -eux

. ci/vars

# Update Cargo.toml
sed -i "0,/^version = .*/{s//version = \"$1\"/}" ./*/Cargo.toml
sed -i "0,/^glslt = { version/{s/glslt = { version = [^,]*/glslt = { version = \"$1\"/}" ./*/Cargo.toml

# Update lockfile
cargo update -p glslt
