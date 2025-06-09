#!/bin/bash
set -e -x

curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
export PATH="$HOME/.cargo/bin:$PATH"

# Maturin builds with musl
rustup target add x86_64-unknown-linux-musl

/opt/python/cp311*/bin/pip install -U maturin==0.13.6
export MATURIN=/opt/_internal/cpython-3.11*/bin/maturin

# Switch to source dir
cd /io/glslt

# Compile wheels
build_pybin () {
    export PYTHON_SYS_EXECUTABLE="$PYBIN/python"
    export PYTHON_LIB=$(${PYBIN}/python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
    export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB"
    $MATURIN build --manylinux=2014 --strip --features python --release -i $PYTHON_SYS_EXECUTABLE "$@"
}

# Build for previous versions of Python
for PYBIN in /opt/python/cp{310,311,312,313}*/bin; do
    build_pybin
done

# Build for current version
PYBIN=$(echo /opt/python/cp310*/bin) build_pybin --sdist

# We're building in Docker but we want outside to access the wheels directory
chmod -R o+rwX /io/target
