jsonrepair (Rust)
=================

[![Crates.io](https://img.shields.io/crates/v/jsonrepair.svg)](https://crates.io/crates/jsonrepair)
[![Documentation](https://docs.rs/jsonrepair/badge.svg)](https://docs.rs/jsonrepair)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Fast, low‑dependency JSON repair for Rust. Turn “almost JSON” into valid JSON safely, with non‑streaming, streaming, and writer‑oriented APIs.

Overview
- Minimal deps: `memchr` + `thiserror` (+ optional `serde`/`serde_json`). No heavy parser frameworks.
- Performance‑focused: `memchr` fast paths in syntax‑safe states, ASCII fast path, careful byte→char mapping.
- Robust streaming: multi‑chunk stability, NDJSON aggregation, tolerant JSONP/fenced wrapping.
- Ergonomic writer API: produce output while parsing to reduce peak memory.

Repairs Covered
- Comments: `//`, `/* ... */`, and optional `#`.
- Unquoted keys/strings; single quotes → double quotes; smart quotes → `"`.
- Missing commas/colons; unclosed brackets/braces; extra/trailing closers.
- Fenced code blocks: ```json ... ``` and plain ``` ... ``` wrappers.
- JSONP wrappers: `name(...)` and tolerant `name ( ... )`.
- String concatenation: `"a" + "b"`.
- Regex literals: `/re+/` → JSON string.
- Keywords: Python `True`/`False`/`None` → `true`/`false`/`null`; `undefined` → `null`.
- Leading zeros policy: keep number or quote as string.
- NDJSON: multiple values → JSON array (non‑streaming) or aggregate on flush (streaming).

Install
Add to `Cargo.toml`:
```toml
[dependencies]
jsonrepair = "0.1"
```

Quick Start
- Non‑streaming
```rust
use jsonrepair::{Options, repair_to_string};

let s = "{'a': 1, b: 'x', /*comment*/ c: /re+/ }";
let out = repair_to_string(s, &Options::default())?;
let v: serde_json::Value = serde_json::from_str(&out)?;
assert_eq!(v["a"], 1);
```

- Streaming (chunked input)
```rust
use jsonrepair::{Options, StreamRepairer};

let mut r = StreamRepairer::new(Options::default());
let mut outs = Vec::new();
for chunk in ["callback(", "```json\n", "{a:1}", "\n```", ");\n"].iter() {
    let s = r.push(chunk)?; if !s.is_empty() { outs.push(s); }
}
let tail = r.flush()?; if !tail.is_empty() { outs.push(tail); }
assert_eq!(outs.len(), 1);
assert_eq!(serde_json::from_str::<serde_json::Value>(&outs[0])?, serde_json::json!({"a":1}));
```

- Streaming writer (parse‑while‑write)
```rust
use jsonrepair::{Options, repair_to_writer_streaming};
use std::fs::File;
use std::io::BufWriter;

let mut f = BufWriter::new(File::create("out.json")?);
repair_to_writer_streaming("{a:1, items: [1 /*c*/, 2, 3]}", &Options::default(), &mut f)?;
```

APIs (Library)
- `repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError>`
- `repair_to_writer(input: &str, opts: &Options, writer: &mut impl Write)`
- `repair_to_writer_streaming(input: &str, opts: &Options, writer: &mut impl Write)`
- Streaming helper
  - `StreamRepairer::new(opts: Options)`
  - `push(&mut self, chunk: &str) -> Result<String, RepairError>`
  - `flush(&mut self) -> Result<String, RepairError>`
  - `push_to_writer/flush_to_writer` for incremental writes

CLI
- Install
  - Local: `cargo install --path .`
  - crates.io: `cargo install jsonrepair`
- Usage (alias: `jr`)
```
jsonrepair [OPTIONS] [INPUT]

INPUT: optional input file. When omitted, reads from stdin.

Options:
  -o, --output FILE         Write output to FILE (default stdout)
      --in-place            Overwrite INPUT file (implies non-streaming)
      --stream              Stream while parsing (lower memory)
      --chunk-size BYTES    Chunk size for streaming (default 65536)
      --ndjson-aggregate    Aggregate NDJSON values into a single array (streaming)
      --pretty              Pretty-print output (non-streaming path)
      --ensure-ascii        Escape non-ASCII as \uXXXX
      --no-python-keywords  Disable Python True/False/None normalization
      --no-undefined-null   Disable undefined -> null repair
      --no-fence            Disable fenced code block stripping
      --no-hash-comments    Disable # line comment tolerance
      --no-nonfinite-null   Disable NaN/Infinity -> null normalization
      --leading-zero POLICY Keep|Quote (default Keep)
  -h, --help                Show help
```
- Tips
  - Use `--stream` for very large inputs to reduce peak memory. Tune `--chunk-size` for I/O.
  - `--ndjson-aggregate` collects multiple values into a single JSON array in streaming mode.
  - `--pretty` requires the `serde` feature (enabled by default).

Options (high level)
- `tolerate_hash_comments: bool` — allow `#` outside strings (default: true)
- `repair_undefined: bool` — convert `undefined` to `null` (default: true)
- `leading_zero_policy: LeadingZeroPolicy` — `KeepAsNumber` | `QuoteAsString` (default: `KeepAsNumber`)
- `fenced_code_blocks: bool` — strip ``` fences (default: true)
- `logging: bool` — enable repair log (use `repair_to_string_with_log`)
- `allow_python_keywords: bool` — normalize `True`/`False`/`None` (default: true)
- `ensure_ascii: bool` — escape non-ASCII as `\uXXXX` (default: false)
- `log_context_window: usize` — context window for log snippets (default: 10)
- `log_json_path: bool` — attach JSON path to logs (default: false)
- `normalize_js_nonfinite: bool` — normalize `NaN`/`Infinity`/`-Infinity` to `null` (default: true)
- `stream_ndjson_aggregate: bool` — aggregate streaming NDJSON values into a single array on `flush()` (default: false)

Logging example
```rust
use jsonrepair::{Options, repair_to_string_with_log};

let mut opts = Options::default();
opts.log_context_window = 12;
opts.log_json_path = true;

let (_out, log) = repair_to_string_with_log("[1, 2 /*c*/, 3]", &opts)?;
for e in log { println!("pos={} path={:?} msg={} ctx={}", e.position, e.path, e.message, e.context); }
```

Performance & Benchmarks
- Fast paths: `memchr` jumps only in syntax‑safe states; UTF‑8/Unicode safe; ASCII fast path.
- **Quick Start**: `python scripts/run_benchmarks.py [profile]` (cross-platform)
  - **Profiles**:
    - `quick` - Fast iteration for development (~1-2 min)
    - `standard` - Balanced accuracy and speed (~3-5 min) **[DEFAULT]**
    - `heavy` - Maximum accuracy for official benchmarks (~8-12 min)
    - `custom` - Use command-line args for custom configuration
  - **Examples**:
    - `python scripts/run_benchmarks.py quick` - Quick development test
    - `python scripts/run_benchmarks.py standard` - Standard benchmark
    - `python scripts/run_benchmarks.py heavy` - High-accuracy benchmark
  - **Platform-specific launchers**:
    - Windows: `scripts\run_benchmarks.bat [profile]`
    - Linux/macOS: `./scripts/run_benchmarks.sh [profile]`
  - **Outputs**: `python_bench.json` (raw data) and `docs/bench_table.md` (comparison table)
- **Manual**:
  - Python: `pip install json_repair` then `python scripts/py_bench.py --min-bytes 1048576 --target-sec 1.0 --warmup 3 > python_bench.json`
  - Rust: `export JR_MIN_BYTES=1048576 JR_MEAS_SEC=3 JR_WARMUP_SEC=1 JR_SAMPLE_SIZE=5` then `cargo bench --bench container_bench`
  - Aggregate: `python scripts/aggregate_bench.py > docs/bench_table.md`
- **Notes**:
  - Throughput (MiB/s): higher is better. Mean time (s): lower is better.
  - Test data generators are verified to match between Python and Rust implementations.
  - If Criterion warns about sample completion, try `quick` profile or increase `JR_MEAS_SEC`.
  - See `docs/benchmark-python.md` for details, case matrix, and fairness guidelines.

References
- We drew inspiration from and aim for practical parity with:
  - jsonrepair (TypeScript): https://github.com/josdejong/jsonrepair
  - json_repair (Python): https://github.com/mangiucugna/json_repair
- This crate adapts many repair rules and test ideas to a Rust‑centric design focused on performance and streaming.

License
- MIT or Apache‑2.0
