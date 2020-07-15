#!/bin/bash
set -e -x

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
export PATH="$HOME/.cargo/bin:$PATH"

/opt/python/cp37*/bin/pip install -U maturin
export MATURIN=/opt/_internal/cpython-3.7*/bin/maturin

# Switch to source dir
cd /io/glslt

# Compile wheels
for PYBIN in /opt/python/cp{35,36,37,38,39}*/bin; do
    export PYTHON_SYS_EXECUTABLE="$PYBIN/python"
    export PYTHON_LIB=$(${PYBIN}/python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
    export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB"
    $MATURIN build --strip --cargo-extra-args='--features python' --release -i $PYTHON_SYS_EXECUTABLE
done

# We're building in Docker but we want outside to access the wheels directory
chmod 0777 /io/target/wheels
