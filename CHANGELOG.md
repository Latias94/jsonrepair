## [Unreleased]

- Housekeeping and documentation improvements.
- Internal performance experiments (no API changes planned).

## [0.1.0] - 2025-10-19

Initial release. This crate aims to provide a pragmatic, fast, and low-dependency "JSON repair" utility for Rust.

Highlights
- Non-streaming repair that handles common "almost JSON" inputs: comments, unquoted keys/strings, single quotes, regex literals, JSONP wrappers, fenced code blocks, string concatenation, Python keywords, `undefined -> null`, `NaN/Infinity -> null`, and NDJSON aggregation.
- Streaming APIs: chunked `StreamRepairer` plus a writer-oriented variant that emits output as values/containers complete.
- Performance-minded implementation using `memchr` fast paths in syntax-safe states, with ASCII fast path and adjustable logging context.
- CLI binaries: `jsonrepair` (main) and `jr` (alias).

Notes for early adopters
- API surface may evolve based on feedback, especially around streaming/writer ergonomics.
- Some advanced logging features (e.g., path tracing in streaming) are intentionally minimal for 0.1.0.
- Benchmark coverage exists but will continue to grow; results can vary by corpus and platform.
- Please file issues for gaps, edge cases, and API suggestions.

