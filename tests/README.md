# C API Tests

This directory contains tests for the C FFI (Foreign Function Interface) of jsonrepair.

## Test Files

- **`ffi_tests.rs`** - Rust integration tests for the C API
- **`c_api_test.c`** - Native C tests
- **`Makefile`** - Build script for C tests
- **`run_tests.sh`** - Automated test runner (Linux/macOS)
- **`run_tests.bat`** - Automated test runner (Windows)

## Running Tests

### Quick Start

The test scripts can be run from either the **project root** or the **tests/** directory.

**From project root (recommended):**

Linux/macOS:
```bash
./tests/run_tests.sh
```

Windows:
```cmd
tests\run_tests.bat
```

**From tests/ directory:**

Linux/macOS:
```bash
cd tests
./run_tests.sh
```

Windows:
```cmd
cd tests
run_tests.bat
```

The scripts will automatically detect which directory you're in and adjust paths accordingly.

### Manual Testing

#### 1. Rust FFI Tests

```bash
cargo test --features c-api --test ffi_tests
```

These tests verify the C API from Rust, ensuring:
- Memory safety
- Correct function signatures
- Proper error handling
- Option setters work correctly

#### 2. C Native Tests

```bash
# Build the library
cargo build --release --features c-api

# Regenerate C header
cbindgen --config cbindgen.toml --crate jsonrepair --output include/jsonrepair.h

# Compile and run C tests
cd tests
make test
```

These tests verify the C API from actual C code, ensuring:
- C header is correct
- Library can be linked from C
- API works as documented
- No memory leaks

## Test Coverage

### Rust FFI Tests (15 tests)

1. ✅ `test_simple_repair` - Basic JSON repair
2. ✅ `test_null_input` - NULL pointer handling
3. ✅ `test_options_lifecycle` - Options creation/destruction
4. ✅ `test_repair_with_options` - Repair with custom options
5. ✅ `test_error_handling` - Error reporting
6. ✅ `test_streaming_basic` - Streaming API basics
7. ✅ `test_python_keywords` - Python keyword conversion
8. ✅ `test_hash_comments` - Hash comment handling
9. ✅ `test_fenced_code_blocks` - Markdown code block extraction
10. ✅ `test_undefined_repair` - JavaScript undefined handling
11. ✅ `test_normalize_nonfinite` - NaN/Infinity normalization
12. ✅ `test_version` - Version string retrieval
13. ✅ `test_streaming_with_error` - Streaming with error tracking
14. ✅ `test_multiple_repairs` - Multiple sequential repairs
15. ✅ `test_new_options` - New option setters

### C Native Tests (13 tests)

1. ✅ `test_version` - Version info
2. ✅ `test_simple_repair` - Basic repair
3. ✅ `test_null_input` - NULL handling
4. ✅ `test_with_options` - Options API
5. ✅ `test_error_handling` - Error tracking
6. ✅ `test_streaming_basic` - Streaming basics
7. ✅ `test_streaming_multiple_values` - Multiple values
8. ✅ `test_python_keywords` - Python keywords
9. ✅ `test_hash_comments` - Hash comments
10. ✅ `test_fenced_code_blocks` - Fenced blocks
11. ✅ `test_undefined_repair` - Undefined repair
12. ✅ `test_normalize_nonfinite` - Non-finite numbers
13. ✅ `test_complex_repair` - Complex scenarios

## API Coverage

### Core Functions
- ✅ `jsonrepair_repair()` - Simple repair
- ✅ `jsonrepair_free()` - Memory cleanup
- ✅ `jsonrepair_version()` - Version info

### Options API
- ✅ `jsonrepair_options_new()` - Create options
- ✅ `jsonrepair_options_free()` - Free options
- ✅ `jsonrepair_options_set_ensure_ascii()` - ASCII escaping
- ✅ `jsonrepair_options_set_allow_python_keywords()` - Python keywords
- ✅ `jsonrepair_options_set_tolerate_hash_comments()` - Hash comments
- ✅ `jsonrepair_options_set_repair_undefined()` - Undefined repair
- ✅ `jsonrepair_options_set_fenced_code_blocks()` - Fenced blocks
- ✅ `jsonrepair_options_set_normalize_js_nonfinite()` - Non-finite numbers
- ✅ `jsonrepair_options_set_stream_ndjson_aggregate()` - NDJSON aggregation
- ✅ `jsonrepair_options_set_logging()` - Logging
- ✅ `jsonrepair_options_set_number_tolerance_leading_dot()` - Leading dot
- ✅ `jsonrepair_options_set_number_tolerance_trailing_dot()` - Trailing dot
- ✅ `jsonrepair_options_set_python_style_separators()` - Python separators
- ✅ `jsonrepair_options_set_aggressive_truncation_fix()` - Truncation fix

### Advanced API
- ✅ `jsonrepair_repair_with_options()` - Repair with options
- ✅ `jsonrepair_repair_ex()` - Repair with error details

### Streaming API
- ✅ `jsonrepair_stream_new()` - Create stream
- ✅ `jsonrepair_stream_free()` - Free stream
- ✅ `jsonrepair_stream_push()` - Push chunk
- ✅ `jsonrepair_stream_flush()` - Flush stream
- ✅ `jsonrepair_stream_push_ex()` - Push with error
- ✅ `jsonrepair_stream_flush_ex()` - Flush with error

## Memory Safety

All tests are designed to verify:
- No memory leaks (all allocated memory is freed)
- No use-after-free
- No double-free
- Proper NULL pointer handling

### Running with Valgrind (Linux/macOS)

```bash
# Build tests
make

# Run with valgrind
valgrind --leak-check=full --show-leak-kinds=all ./c_api_test
```

Expected output: "All heap blocks were freed -- no leaks are possible"

## Continuous Integration

These tests should be run in CI/CD:

```yaml
# Example GitHub Actions
- name: Test C API
  run: |
    cargo test --features c-api --test ffi_tests
    cd tests && make test
```

## Troubleshooting

### Library Not Found

**Linux/macOS:**
```bash
export LD_LIBRARY_PATH=../target/release:$LD_LIBRARY_PATH
./c_api_test
```

**Windows:**
```cmd
set PATH=%PATH%;..\target\release
c_api_test.exe
```

### Header Not Found

Make sure to regenerate the header after code changes:
```bash
cbindgen --config cbindgen.toml --crate jsonrepair --output include/jsonrepair.h
```

### Compilation Errors

Make sure you have:
- GCC or Clang installed
- Rust toolchain installed
- cbindgen installed (`cargo install cbindgen`)

## Adding New Tests

### Rust FFI Test

Add to `ffi_tests.rs`:
```rust
#[test]
fn test_my_feature() {
    unsafe {
        // Your test code
    }
}
```

### C Test

Add to `c_api_test.c`:
```c
void test_my_feature() {
    TEST("my feature");
    
    // Your test code
    
    PASS();
}

// Add to main():
test_my_feature();
```

## See Also

- [C API Design](../docs/c_api_design.md)
- [C Example](../examples/c_example/)
- [Go Example](../examples/go_example/)

