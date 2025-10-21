#!/bin/bash
# Quick C API test script
# This script runs the same tests that CI runs

set -e

echo "================================="
echo "C API Test Script"
echo "================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    echo -e "${YELLOW}Running: ${test_name}${NC}"
    if eval "$test_cmd"; then
        echo -e "${GREEN}✓ ${test_name} passed${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ ${test_name} failed${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
    echo ""
}

# Check if cbindgen is installed
if ! command -v cbindgen &> /dev/null; then
    echo -e "${YELLOW}cbindgen not found, installing...${NC}"
    cargo install cbindgen --version 0.27.0
fi

# 1. Build library
run_test "Build library with c-api feature" \
    "cargo build --release --features c-api"

# 2. Generate C header
run_test "Generate C header" \
    "cbindgen --config cbindgen.toml --crate jsonrepair --output include/jsonrepair.h"

# 3. Run Rust FFI tests
run_test "Rust FFI tests" \
    "cargo test --features c-api --test ffi_tests"

# 4. Check if GCC is available
if command -v gcc &> /dev/null; then
    echo -e "${GREEN}GCC found, running C native tests${NC}"
    
    # Compile C tests
    run_test "Compile C tests" \
        "cd tests && make clean && make"
    
    # Run C tests
    run_test "Run C tests" \
        "cd tests && make test"
    
    # Check if valgrind is available
    if command -v valgrind &> /dev/null; then
        echo -e "${GREEN}Valgrind found, running memory leak tests${NC}"
        run_test "Valgrind memory leak test" \
            "cd tests && valgrind --leak-check=full --error-exitcode=1 ./c_api_test 2>&1 | tail -20"
    else
        echo -e "${YELLOW}Valgrind not found, skipping memory leak tests${NC}"
    fi
else
    echo -e "${YELLOW}GCC not found, skipping C native tests${NC}"
    echo "Install GCC to run C tests:"
    echo "  Ubuntu/Debian: sudo apt-get install gcc"
    echo "  macOS: xcode-select --install"
    echo "  Windows: Install MinGW or MSVC"
fi

# Summary
echo ""
echo "================================="
echo "Test Summary"
echo "================================="
echo -e "Tests passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests failed: ${RED}${TESTS_FAILED}${NC}"
echo "================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed!${NC}"
    exit 1
fi

