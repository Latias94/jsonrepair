@echo off
REM Run all C API tests on Windows
REM Can be run from project root or tests/ directory

echo =================================
echo Running C API Tests
echo =================================
echo.

REM Detect if we're in the tests directory or project root
if exist "Cargo.toml" (
    set "PROJECT_ROOT=%CD%"
    set "TESTS_DIR=%CD%\tests"
) else if exist "..\Cargo.toml" (
    set "PROJECT_ROOT=%CD%\.."
    set "TESTS_DIR=%CD%"
) else (
    echo Error: Cannot find Cargo.toml. Please run from project root or tests/ directory.
    exit /b 1
)

echo Project root: %PROJECT_ROOT%
echo Tests directory: %TESTS_DIR%
echo.

echo 1. Building Rust library with c-api feature...
cd "%PROJECT_ROOT%"
cargo build --release --features c-api
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

echo.
echo 2. Regenerating C header...
cbindgen --config cbindgen.toml --crate jsonrepair --output include/jsonrepair.h
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

echo.
echo 3. Running Rust FFI tests...
cargo test --features c-api --test ffi_tests
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

echo.
echo 4. Compiling C tests...
cd "%TESTS_DIR%"
make clean
make
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

echo.
echo 5. Running C tests...
make test
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

echo.
echo =================================
echo All tests completed successfully!
echo =================================
