## [Unreleased]

## [0.1.0] - TBD

Initial release. A pragmatic, fast, and low-dependency JSON repair utility for Rust with multi-language bindings.

### Core Features

- **Non-streaming repair**: Handles common "almost JSON" inputs
  - Comments (`//` and `/* */`), hash comments (`#`)
  - Unquoted keys and string values
  - Single quotes instead of double quotes
  - Trailing commas
  - JSONP wrappers and fenced code blocks (` ```json ... ``` `)
  - Python keywords (`True`/`False`/`None` → `true`/`false`/`null`)
  - JavaScript non-finite values (`undefined`/`NaN`/`Infinity` → `null`)
  - NDJSON aggregation to array
  - Incomplete JSON (missing closing brackets)

- **Streaming APIs**: Process large files or chunked input
  - `StreamRepairer`: Push chunks, get output when complete values are produced
  - Writer-based API: `repair_to_writer()`, `repair_to_writer_streaming()`
  - Configurable buffering and output strategies

- **Performance-minded implementation**
  - Zero-copy architecture using `&str` slicing
  - Hand-written recursive descent parser
  - `memchr` fast paths for syntax scanning
  - Minimal memory allocations
  - 1.5-3.4x faster than alternatives for medium-large JSON
  - 4x less memory usage

- **CLI tools**: `jsonrepair` and `jr` (alias)
  - Repair from stdin, files, or arguments
  - Streaming mode for large files
  - Configurable options via flags

### Language Bindings

- **Python (PyO3)**: Native Python package with full `json_repair` compatibility
  - Functions: `repair_json()`, `loads()`, `load()`, `from_file()`
  - Advanced: `RepairOptions` class with property access, `StreamRepairer` for chunked processing
  - Logging: `repair_json_with_log()`, `loads_with_log()`, `from_file_with_log()`
  - Type hints: Complete `.pyi` stub file for IDE support
  - Performance: 20-400x faster than pure Python implementations
  - Zero Python dependencies, supports Python 3.8-3.14
  - See `python/` directory for documentation and examples

- **C/C++ (FFI)**: 26 functions for cross-language integration
  - Non-streaming: `jsonrepair_repair()`, `jsonrepair_repair_to_string()`, etc.
  - Streaming: `jsonrepair_stream_repairer_new()`, `jsonrepair_stream_repairer_push()`, etc.
  - Options: `jsonrepair_options_new()`, `jsonrepair_options_set_*()` functions
  - Memory management: `jsonrepair_free_string()`, `jsonrepair_free_error()`, etc.
  - Header generation via `cbindgen` (enable with `c-api` feature)
  - See `examples/c_example/` for usage examples

- **Go**: Example bindings using cgo (see `examples/go_example/`)

### Configuration

- **Features**:
  - `serde` (default): Enable `serde_json::Value` parsing
  - `logging` (default): Enable repair operation logging
  - `c-api`: Enable C header generation with cbindgen

- **RepairOptions**: Fine-grained control over repair behavior
  - Comment handling, keyword normalization, code fence extraction
  - Number tolerance (leading/trailing dots, incomplete exponents)
  - Logging verbosity and context window
  - NDJSON aggregation mode

### Development & CI

- **CI/CD workflows**: Automated testing and release
  - Main CI: Rust tests, clippy, formatting checks across multiple platforms
  - C API tests: Build and run C examples
  - Release workflow: Automated crates.io publishing

- **Comprehensive test suite**:
  - 100+ compatibility tests with Python `json_repair`
  - Fuzz testing for streaming and edge cases
  - Real-world LLM output benchmarks

### Documentation

- Clear, concise README focused on practical usage
- Performance comparison tables with honest trade-off analysis
- Recommendations for alternative libraries where appropriate
- Comprehensive examples for all language bindings
- API documentation with examples

### Notes for Early Adopters

- API surface may evolve based on feedback, especially around streaming/writer ergonomics
- Some advanced logging features (e.g., path tracing in streaming) are intentionally minimal for 0.1.0
- Benchmark coverage will continue to grow; results vary by corpus and platform
- Please file issues for gaps, edge cases, and API suggestions

### Acknowledgments

Inspired by [json_repair](https://github.com/mangiucugna/json_repair) by mangiucugna
