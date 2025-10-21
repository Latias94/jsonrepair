#!/bin/bash
# Run all C API tests
# Can be run from project root or tests/ directory

set -e

echo "================================="
echo "Running C API Tests"
echo "================================="
echo ""

# Detect if we're in the tests directory or project root
if [ -f "Cargo.toml" ]; then
    PROJECT_ROOT="$(pwd)"
    TESTS_DIR="$(pwd)/tests"
elif [ -f "../Cargo.toml" ]; then
    PROJECT_ROOT="$(cd .. && pwd)"
    TESTS_DIR="$(pwd)"
else
    echo "Error: Cannot find Cargo.toml. Please run from project root or tests/ directory."
    exit 1
fi

echo "Project root: $PROJECT_ROOT"
echo "Tests directory: $TESTS_DIR"
echo ""

# Build the library
echo "1. Building Rust library with c-api feature..."
cd "$PROJECT_ROOT"
cargo build --release --features c-api

echo ""
echo "2. Regenerating C header..."
cbindgen --config cbindgen.toml --crate jsonrepair --output include/jsonrepair.h

echo ""
echo "3. Running Rust FFI tests..."
cargo test --features c-api --test ffi_tests

echo ""
echo "4. Compiling C tests..."
cd "$TESTS_DIR"
make clean
make

echo ""
echo "5. Running C tests..."
make test

echo ""
echo "================================="
echo "All tests completed successfully!"
echo "================================="
