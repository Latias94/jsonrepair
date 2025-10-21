# C API Examples

This directory contains examples of using the jsonrepair library from C.

## Examples

- **`basic.c`** - Basic usage examples (simple repair, options, error handling, streaming)
- **`advanced.c`** - Advanced features (all options, combined usage, NDJSON streaming)

## Building

### Prerequisites

- Rust toolchain (for building the library)
- C compiler (gcc, clang, or MSVC)
- Make (optional, for using Makefile)

### Build the Library

First, build the Rust library with the `c-api` feature:

```bash
cd ../..
cargo build --release --features c-api
```

This will:
1. Build the Rust library as a dynamic library (`.so`, `.dylib`, or `.dll`)
2. Generate the C header file at `include/jsonrepair.h`

### Compile Examples

#### Using Make

```bash
cd examples/c_example
make
```

#### Manual Compilation

**Linux/macOS:**
```bash
gcc -o basic basic.c -I../../include -L../../target/release -ljsonrepair
```

**Windows (MSVC):**
```cmd
cl basic.c /I..\..\include /link /LIBPATH:..\..\target\release jsonrepair.lib
```

## Running

### Run All Examples

```bash
make run
```

### Run Individual Examples

**Linux/macOS:**

```bash
# Basic example
LD_LIBRARY_PATH=../../target/release ./basic

# Advanced example
LD_LIBRARY_PATH=../../target/release ./advanced
```

Or use Make:

```bash
make run-basic
make run-advanced
```

**Windows:**

```cmd
set PATH=%PATH%;..\..\target\release
basic.exe
advanced.exe
```

## API Overview

### Simple API

```c
#include "jsonrepair.h"

// Repair with default options
char* repaired = jsonrepair_repair("{a:1}");
if (repaired) {
    printf("%s\n", repaired);
    jsonrepair_free(repaired);
}
```

### With Options

```c
JsonRepairOptions* opts = jsonrepair_options_new();
jsonrepair_options_set_ensure_ascii(opts, true);

char* repaired = jsonrepair_repair_with_options("{name: '统一码'}", opts);
jsonrepair_options_free(opts);
jsonrepair_free(repaired);
```

### Error Handling

```c
JsonRepairError error = {0};
char* repaired = jsonrepair_repair_ex("{a:1", NULL, &error);

if (!repaired) {
    fprintf(stderr, "Error %d at position %zu: %s\n",
            error.code, error.position, error.message);
    free(error.message);
}
```

### Streaming

```c
JsonRepairStream* stream = jsonrepair_stream_new(NULL);

char* out = jsonrepair_stream_push(stream, "{a:");
if (out) jsonrepair_free(out);

out = jsonrepair_stream_push(stream, "1}");
if (out) {
    printf("%s\n", out);
    jsonrepair_free(out);
}

char* tail = jsonrepair_stream_flush(stream);
if (tail) jsonrepair_free(tail);

jsonrepair_stream_free(stream);
```

## Memory Management

**Important:** All strings returned by the library must be freed with `jsonrepair_free()`.

- `jsonrepair_repair()` → `jsonrepair_free()`
- `jsonrepair_stream_push()` → `jsonrepair_free()`
- `jsonrepair_stream_flush()` → `jsonrepair_free()`
- `error.message` → `free()` (standard C free)

## Thread Safety

- ✅ `jsonrepair_repair*()` functions are thread-safe
- ✅ `JsonRepairOptions` is immutable and thread-safe
- ❌ `JsonRepairStream` is NOT thread-safe (use one per thread)

## Available Options

### Basic Options

```c
void jsonrepair_options_set_ensure_ascii(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_allow_python_keywords(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_tolerate_hash_comments(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_repair_undefined(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_fenced_code_blocks(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_normalize_js_nonfinite(JsonRepairOptions* opts, bool value);
```

### Advanced Options

```c
void jsonrepair_options_set_stream_ndjson_aggregate(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_logging(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_number_tolerance_leading_dot(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_number_tolerance_trailing_dot(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_python_style_separators(JsonRepairOptions* opts, bool value);
void jsonrepair_options_set_aggressive_truncation_fix(JsonRepairOptions* opts, bool value);
```

See `advanced.c` for usage examples of all options.

## Error Codes

```c
#define JSONREPAIR_OK 0
#define JSONREPAIR_ERR_UNEXPECTED_END 1
#define JSONREPAIR_ERR_UNEXPECTED_CHAR 2
#define JSONREPAIR_ERR_OBJECT_KEY_EXPECTED 3
#define JSONREPAIR_ERR_COLON_EXPECTED 4
#define JSONREPAIR_ERR_INVALID_UNICODE 5
#define JSONREPAIR_ERR_PARSE 6
```

## See Also

- [Go Example](../go_example/) - Using the C API from Go
- [API Design Document](../../docs/c_api_design.md) - Detailed API design
- [Main README](../../README.md) - Rust API documentation
