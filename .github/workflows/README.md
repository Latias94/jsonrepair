# GitHub Actions Workflows

This directory contains CI/CD workflows for the jsonrepair project.

## Workflows

### 1. `ci.yml` - Main CI Pipeline

**Triggers:** Push to `main`/`develop`, Pull Requests

**Jobs:**
- **Test** - Run all Rust tests including C API FFI tests
- **Docs** - Build documentation

**Platforms:** Ubuntu (Linux)

### 2. `c-api-tests.yml` - C API Comprehensive Tests

**Triggers:** Push to `main`/`master`/`develop`, Pull Requests

**Jobs:**

#### `rust-ffi-tests`
- Runs Rust FFI tests on Ubuntu, Windows, macOS
- Validates C API from Rust side
- Generates and uploads C headers

#### `c-native-tests`
- Compiles and runs native C tests on Ubuntu and macOS
- Uses GCC to compile C test suite
- Runs Valgrind memory leak detection on Ubuntu

#### `windows-msvc`
- Tests C API on Windows with MSVC compiler
- Compiles C tests with `cl.exe`
- Validates Windows DLL linking

#### `build-libraries`
- Cross-compiles libraries for multiple targets:
  - `x86_64-unknown-linux-gnu` (Linux glibc)
  - `x86_64-unknown-linux-musl` (Linux musl)
  - `x86_64-apple-darwin` (macOS Intel)
  - `aarch64-apple-darwin` (macOS Apple Silicon)
  - `x86_64-pc-windows-msvc` (Windows)
- Packages libraries with headers
- Uploads as artifacts

#### `test-summary`
- Aggregates all test results
- Fails if any test job fails

### 3. `release.yml` - Release Pipeline

**Triggers:** Git tags

**Jobs:**
- Build release binaries
- Publish to crates.io
- Create GitHub releases

## CI Status Badges

Add these to your README.md:

```markdown
[![CI](https://github.com/YOUR_USERNAME/jsonrepair/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/jsonrepair/actions/workflows/ci.yml)
[![C API Tests](https://github.com/YOUR_USERNAME/jsonrepair/workflows/C%20API%20Tests/badge.svg)](https://github.com/YOUR_USERNAME/jsonrepair/actions/workflows/c-api-tests.yml)
```

## Local Testing

Before pushing, run these locally:

```bash
# Run all tests
cargo test --all-features

# Run C API tests
cargo test --features c-api --test ffi_tests

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Build C tests (Linux/macOS)
cd tests && make test
```

## Artifacts

The `build-libraries` job produces downloadable artifacts:

- `library-x86_64-unknown-linux-gnu` - Linux (glibc) library
- `library-x86_64-unknown-linux-musl` - Linux (musl) library
- `library-x86_64-apple-darwin` - macOS Intel library
- `library-aarch64-apple-darwin` - macOS ARM library
- `library-x86_64-pc-windows-msvc` - Windows library

Each artifact contains:
```
dist/
├── include/
│   └── jsonrepair.h
└── lib/
    ├── libjsonrepair.so (or .dylib/.dll)
    └── libjsonrepair.a (static library)
```

## Caching

All workflows use caching to speed up builds:

- Cargo registry cache
- Cargo build cache
- Rust toolchain cache

## Required Secrets

No secrets are required for CI tests.

For releases, you may need:
- `CARGO_REGISTRY_TOKEN` - For publishing to crates.io

## Troubleshooting

### Tests fail on Windows

Make sure MSVC is properly set up. The workflow uses `ilammy/msvc-dev-cmd@v1` to configure the environment.

### C tests fail to compile

Ensure GCC is available on Linux/macOS. The workflow installs it automatically.

### Valgrind reports leaks

Check the C test code for missing `jsonrepair_free()` calls.

### Cross-compilation fails

Some targets may require additional setup. Check the Rust target documentation.

## Adding New Tests

1. Add Rust tests to `tests/ffi_tests.rs`
2. Add C tests to `tests/c_api_test.c`
3. Update `tests/c_api_test.c` main function to call new tests
4. CI will automatically run new tests

## Performance

Typical CI run times:
- `ci.yml`: ~3-5 minutes
- `c-api-tests.yml`: ~15-20 minutes (parallel jobs)
- `build-libraries`: ~10-15 minutes per target

## See Also

- [C API Tests Documentation](../../tests/README.md)
- [C API Design](../../docs/c_api_design.md)
- [C API Extended Features](../../docs/c_api_extended.md)

