# jsonrepair

[![Crates.io](https://img.shields.io/crates/v/jsonrepair.svg)](https://crates.io/crates/jsonrepair)
[![Documentation](https://docs.rs/jsonrepair/badge.svg)](https://docs.rs/jsonrepair)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A fast, low-dependency JSON repair library for Rust. Fixes malformed JSON commonly produced by LLMs and other sources.

## Features

- **Non-streaming API**: Simple string-to-string repair
- **Streaming API**: Process large files or chunked input with low memory usage
- **Writer API**: Stream output while parsing to minimize memory overhead
- **Parse to Value**: Direct conversion to `serde_json::Value` (like Python's `json.loads()`)
- **CLI tool**: Command-line interface (`jsonrepair` / `jr`)
- **Language bindings**: Python (PyO3), C API for C/C++/Go/Java/C#/Node.js/etc.

## Installation

```toml
[dependencies]
jsonrepair = "0.1"
```

## Quick Start

### Basic Repair

```rust
use jsonrepair::{repair_json, Options};

let broken = "{name: 'John', age: 30,}";
let fixed = repair_json(broken, &Options::default())?;
// Result: {"name":"John","age":30}
```

### Parse to Value

```rust
use jsonrepair::loads;

let value = loads("{a: 1, b: 'hello'}", &Options::default())?;
assert_eq!(value["a"], 1);
```

### Streaming (Large Files)

```rust
use jsonrepair::StreamRepairer;

let mut repairer = StreamRepairer::new(Options::default());
for chunk in chunks {
    if let Some(output) = repairer.push(chunk)? {
        // Process output
    }
}
if let Some(final_output) = repairer.flush()? {
    // Process final output
}
```

### Load from File

```rust
use jsonrepair::from_file;

let value = from_file("broken.json", &Options::default())?;
```

## What It Fixes

- **Comments**: `//`, `/* ... */`, `#` (optional)
- **Quotes**: Single quotes → double quotes, unquoted keys/strings
- **Punctuation**: Missing commas/colons, trailing commas, unclosed brackets
- **Wrappers**: Fenced code blocks (```json ... ```), JSONP (`callback(...)`)
- **String concatenation**: `"a" + "b"` → `"ab"`
- **Regex literals**: `/pattern/` → `"/pattern/"`
- **Keywords**: Python `True`/`False`/`None`, JavaScript `undefined`
- **Numbers**: `NaN`/`Infinity` → `null`, leading zeros handling
- **NDJSON**: Multiple values → array (optional aggregation)

## API Reference

### Non-Streaming

```rust
// Repair to string
repair_json(input: &str, opts: &Options) -> Result<String>

// Parse to Value (requires 'serde' feature)
loads(input: &str, opts: &Options) -> Result<serde_json::Value>
load(reader: impl Read, opts: &Options) -> Result<serde_json::Value>
from_file(path: impl AsRef<Path>, opts: &Options) -> Result<serde_json::Value>

// Write to writer
repair_to_writer(input: &str, opts: &Options, writer: &mut impl Write)
```

### Streaming

```rust
let mut repairer = StreamRepairer::new(opts);
repairer.push(chunk: &str) -> Result<Option<String>>
repairer.flush() -> Result<Option<String>>

// Writer variants
repairer.push_to_writer(chunk: &str, writer: &mut impl Write)
repairer.flush_to_writer(writer: &mut impl Write)
```

### Options

```rust
Options {
    tolerate_hash_comments: bool,        // Allow # comments (default: true)
    repair_undefined: bool,              // undefined → null (default: true)
    allow_python_keywords: bool,         // True/False/None (default: true)
    normalize_js_nonfinite: bool,        // NaN/Infinity → null (default: true)
    fenced_code_blocks: bool,            // Strip ``` fences (default: true)
    stream_ndjson_aggregate: bool,       // Aggregate NDJSON (default: false)
    leading_zero_policy: LeadingZeroPolicy, // KeepAsNumber | QuoteAsString
    ensure_ascii: bool,                  // Escape non-ASCII (default: false)
    logging: bool,                       // Enable repair log (default: false)
    // ... more options in docs
}
```

## CLI Usage

Install:
```bash
cargo install jsonrepair
```

Basic usage:
```bash
# From file
jsonrepair input.json -o output.json

# From stdin
echo "{a: 1}" | jsonrepair

# In-place edit
jsonrepair --in-place broken.json

# Streaming mode (low memory)
jsonrepair --stream large_file.json

# Pretty print
jsonrepair --pretty input.json
```

Options:
```
-o, --output FILE       Output file (default: stdout)
--in-place              Overwrite input file
--stream                Streaming mode (low memory)
--ndjson-aggregate      Aggregate NDJSON into array
--pretty                Pretty-print output
--ensure-ascii          Escape non-ASCII characters
--no-python-keywords    Disable Python keyword normalization
--no-undefined-null     Disable undefined → null
--no-fence              Disable fence stripping
--no-hash-comments      Disable # comments
```

## Language Bindings

### Python

Native Python bindings using PyO3:

```bash
pip install jsonrepair
```

```python
import jsonrepair

# Repair and parse
data = jsonrepair.loads("{name: 'John', age: 30}")
# {'name': 'John', 'age': 30}

# Just repair, return string
fixed = jsonrepair.repair_json("{a: 1}")
# '{"a":1}'

# Load from file
data = jsonrepair.from_file('broken.json')

# Streaming API
repairer = jsonrepair.StreamRepairer()
for chunk in chunks:
    if output := repairer.push(chunk):
        process(output)
if final := repairer.flush():
    process(final)
```

See [python/](python/) for complete documentation and examples.

### C/C++

Build with C API:
```bash
cargo build --release --features c-api
```

This generates:
- Header: `include/jsonrepair.h`
- Library: `target/release/libjsonrepair.{so,dylib,dll}`

Example:
```c
#include "jsonrepair.h"

char* fixed = jsonrepair_repair("{a:1}");
printf("%s
", fixed);  // {"a":1}
jsonrepair_free(fixed);
```

See [examples/c_example](examples/c_example/) for complete examples.

### Go

```go
// #cgo LDFLAGS: -L./target/release -ljsonrepair
// #include "include/jsonrepair.h"
import "C"

result := C.jsonrepair_repair(C.CString("{a:1}"))
defer C.jsonrepair_free(result)
fmt.Println(C.GoString(result))
```

See [examples/go_example](examples/go_example/) for complete examples.

### Other Languages

The C API works with any language supporting C FFI:

- **Java**: JNA, JNI
- **C#**: P/Invoke
- **Node.js**: node-ffi, napi
- **Ruby**: FFI gem

## Performance

This library uses a zero-copy architecture (hand-written recursive descent parser with `&str` slicing) optimized for:
- Medium to large JSON (1KB - 1MB+)
- API responses, database exports, config files
- Batch processing, file parsing

For NDJSON streams or heavily commented small objects, consider [llm_json](https://github.com/oramasearch/llm_json) which uses a different architecture optimized for those patterns.

## Engine Notes

- This project ships two engines and lets you choose at runtime:
  - Recursive-descent (default): most stable and fully featured. Supports writer/streaming/NDJSON/logging/JSONPath.
  - LLM scanner: a lightweight scanner optimized for LLM-style outputs. See `docs/LLM_ENGINE.md` for current status, perf and roadmap.

Run benchmarks:
```bash
python scripts/run_benchmarks.py
```

## Related Projects

- **[json_repair (Python)](https://github.com/mangiucugna/json_repair)** - The original Python implementation that inspired this project
- **[llm_json (Rust)](https://github.com/oramasearch/llm_json)** - Alternative Rust implementation, also based on json_repair
- **[jsonrepair (TypeScript)](https://github.com/josdejong/jsonrepair)** - TypeScript implementation by Jos de Jong

## Acknowledgments

This project is inspired by and aims to be compatible with [json_repair](https://github.com/mangiucugna/json_repair) by Stefano Baccianella. We're grateful for the excellent design and comprehensive test suite that made this implementation possible.

## License

MIT or Apache-2.0
