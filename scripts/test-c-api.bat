@echo off
REM Quick C API test script for Windows
REM This script runs the same tests that CI runs

setlocal enabledelayedexpansion

echo =================================
echo C API Test Script (Windows)
echo =================================
echo.

set TESTS_PASSED=0
set TESTS_FAILED=0

REM Check if cbindgen is installed
where cbindgen >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo cbindgen not found, installing...
    cargo install cbindgen --version 0.27.0
    if %ERRORLEVEL% NEQ 0 (
        echo Failed to install cbindgen
        exit /b 1
    )
)

REM 1. Build library
echo.
echo [1/4] Building library with c-api feature...
cargo build --release --features c-api
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Build library
    set /a TESTS_PASSED+=1
) else (
    echo [FAIL] Build library
    set /a TESTS_FAILED+=1
)

REM 2. Generate C header
echo.
echo [2/4] Generating C header...
cbindgen --config cbindgen.toml --crate jsonrepair --output include/jsonrepair.h
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Generate C header
    set /a TESTS_PASSED+=1
) else (
    echo [FAIL] Generate C header
    set /a TESTS_FAILED+=1
)

REM 3. Run Rust FFI tests
echo.
echo [3/4] Running Rust FFI tests...
cargo test --features c-api --test ffi_tests
if %ERRORLEVEL% EQU 0 (
    echo [PASS] Rust FFI tests
    set /a TESTS_PASSED+=1
) else (
    echo [FAIL] Rust FFI tests
    set /a TESTS_FAILED+=1
)

REM 4. Check for C compiler
echo.
echo [4/4] Checking for C compiler...

REM Check for MSVC
where cl >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo MSVC found, compiling C tests...
    cd tests
    cl /W4 /I..\include c_api_test.c /Fe:c_api_test_msvc.exe /link /LIBPATH:..\target\release jsonrepair.dll.lib
    if %ERRORLEVEL% EQU 0 (
        echo [PASS] Compile C tests (MSVC)
        set /a TESTS_PASSED+=1
        
        echo Running C tests...
        set PATH=%PATH%;..\target\release
        c_api_test_msvc.exe
        if %ERRORLEVEL% EQU 0 (
            echo [PASS] Run C tests
            set /a TESTS_PASSED+=1
        ) else (
            echo [FAIL] Run C tests
            set /a TESTS_FAILED+=1
        )
    ) else (
        echo [FAIL] Compile C tests (MSVC)
        set /a TESTS_FAILED+=1
    )
    cd ..
    goto summary
)

REM Check for MinGW GCC
where gcc >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo MinGW GCC found, compiling C tests...
    cd tests
    make clean
    make
    if %ERRORLEVEL% EQU 0 (
        echo [PASS] Compile C tests (GCC)
        set /a TESTS_PASSED+=1
        
        echo Running C tests...
        make test
        if %ERRORLEVEL% EQU 0 (
            echo [PASS] Run C tests
            set /a TESTS_PASSED+=1
        ) else (
            echo [FAIL] Run C tests
            set /a TESTS_FAILED+=1
        )
    ) else (
        echo [FAIL] Compile C tests (GCC)
        set /a TESTS_FAILED+=1
    )
    cd ..
    goto summary
)

echo No C compiler found (MSVC or MinGW GCC)
echo Install Visual Studio or MinGW to run C tests
echo Skipping C native tests...

:summary
echo.
echo =================================
echo Test Summary
echo =================================
echo Tests passed: %TESTS_PASSED%
echo Tests failed: %TESTS_FAILED%
echo =================================

if %TESTS_FAILED% EQU 0 (
    echo All tests passed!
    exit /b 0
) else (
    echo Some tests failed!
    exit /b 1
)

