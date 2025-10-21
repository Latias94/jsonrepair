# Go Example using jsonrepair C API

This example demonstrates how to use the jsonrepair library from Go using cgo.

## Prerequisites

- Go 1.21 or later
- Rust toolchain (for building the library)
- C compiler (gcc, clang, or MSVC)

## Building

### 1. Build the Rust Library

```bash
cd ../..
cargo build --release --features c-api
```

This generates:
- Dynamic library: `target/release/libjsonrepair.so` (Linux), `libjsonrepair.dylib` (macOS), or `jsonrepair.dll` (Windows)
- C header: `include/jsonrepair.h`

### 2. Run the Go Example

**Linux/macOS:**
```bash
cd examples/go_example
LD_LIBRARY_PATH=../../target/release go run main.go
```

**Windows:**
```cmd
cd examples\go_example
set PATH=%PATH%;..\..\target\release
go run main.go
```

## Code Overview

### Simple Repair

```go
func RepairJSON(input string) (string, error) {
    cInput := C.CString(input)
    defer C.free(unsafe.Pointer(cInput))

    cResult := C.jsonrepair_repair(cInput)
    if cResult == nil {
        return "", fmt.Errorf("repair failed")
    }
    defer C.jsonrepair_free(cResult)

    return C.GoString(cResult), nil
}

// Usage
repaired, err := RepairJSON("{a:1, b:'hello'}")
```

### With Options

```go
func RepairJSONWithOptions(input string, ensureASCII bool) (string, error) {
    cInput := C.CString(input)
    defer C.free(unsafe.Pointer(cInput))

    opts := C.jsonrepair_options_new()
    defer C.jsonrepair_options_free(opts)

    C.jsonrepair_options_set_ensure_ascii(opts, C.bool(ensureASCII))

    cResult := C.jsonrepair_repair_with_options(cInput, opts)
    if cResult == nil {
        return "", fmt.Errorf("repair failed")
    }
    defer C.jsonrepair_free(cResult)

    return C.GoString(cResult), nil
}
```

### Streaming API

```go
type StreamRepairer struct {
    stream *C.JsonRepairStream
}

func NewStreamRepairer() *StreamRepairer {
    return &StreamRepairer{
        stream: C.jsonrepair_stream_new(nil),
    }
}

func (s *StreamRepairer) Push(chunk string) (string, error) {
    cChunk := C.CString(chunk)
    defer C.free(unsafe.Pointer(cChunk))

    cResult := C.jsonrepair_stream_push(s.stream, cChunk)
    if cResult == nil {
        return "", nil // No complete value yet
    }
    defer C.jsonrepair_free(cResult)

    return C.GoString(cResult), nil
}

// Usage
stream := NewStreamRepairer()
defer stream.Close()

out, _ := stream.Push("{a:")
out, _ = stream.Push("1}")
tail, _ := stream.Flush()
```

## Memory Management

The example properly manages memory by:

1. **C strings from Go:** Use `C.CString()` and `defer C.free()`
2. **Strings from Rust:** Use `defer C.jsonrepair_free()`
3. **Stream objects:** Call `Close()` or use `defer stream.Close()`

## Building a Go Package

To create a reusable Go package:

```go
package jsonrepair

/*
#cgo LDFLAGS: -ljsonrepair
#include <jsonrepair.h>
*/
import "C"
import "unsafe"

func Repair(input string) (string, error) {
    // ... implementation
}
```

Then users can:

```go
import "github.com/yourname/jsonrepair-go"

result, err := jsonrepair.Repair("{a:1}")
```

## Performance Notes

- cgo calls have overhead (~50-100ns per call)
- For best performance, batch operations or use streaming API
- The Rust library itself is very fast (1.5-3.4x faster than Python alternatives)

## See Also

- [C Example](../c_example/) - Direct C usage
- [API Design](../../docs/c_api_design.md) - Full API documentation
- [Main README](../../README.md) - Rust API

