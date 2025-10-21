mod classify;
pub mod cli;
mod emit;
pub mod error;
pub mod options;
mod parser;
mod repair;
pub mod stream;

#[cfg(feature = "c-api")]
pub mod ffi;

pub use error::{RepairError, RepairErrorKind};
pub use options::{LeadingZeroPolicy, Options};
pub use repair::RepairLogEntry;
pub use stream::StreamRepairer;

use std::io::Write;

// ============================================================================
// Core API - Repair to String
// ============================================================================

/// Repair a potentially invalid JSON string into a valid JSON string.
///
/// This function focuses on common issues like unquoted keys/strings,
/// missing commas/colons, comments, and unclosed brackets/braces.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_to_string, Options};
///
/// let broken = r#"{name: 'John', age: 30,}"#;
/// let repaired = repair_to_string(broken, &Options::default())?;
/// assert_eq!(repaired, r#"{"name":"John","age":30}"#);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError> {
    repair::repair_to_string(input, opts)
}

/// Alias for [`repair_to_string`] - repairs broken JSON and returns a valid JSON string.
///
/// This naming is more intuitive and matches the Python `json_repair` library.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_json, Options};
///
/// let broken = r#"{name: 'John', age: 30,}"#;
/// let repaired = repair_json(broken, &Options::default())?;
/// assert_eq!(repaired, r#"{"name":"John","age":30}"#);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_json(input: &str, opts: &Options) -> Result<String, RepairError> {
    repair_to_string(input, opts)
}

// ============================================================================
// Writer-based API
// ============================================================================

/// Repair a potentially invalid JSON string and write the result into an `io::Write`.
///
/// This avoids an extra copy of the final string when the caller intends to stream to a sink.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_to_writer, Options};
///
/// let broken = r#"{name: 'John'}"#;
/// let mut output = Vec::new();
/// repair_to_writer(broken, &Options::default(), &mut output)?;
/// assert_eq!(output, br#"{"name":"John"}"#);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_to_writer<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    let s = repair::repair_to_string(input, opts)?;
    writer
        .write_all(s.as_bytes())
        .map_err(|e| RepairError::new(RepairErrorKind::Parse(format!("write error: {}", e)), 0))
}

/// Repair a potentially invalid JSON string and stream the output into a writer while parsing.
///
/// This reduces peak memory usage for very large inputs by flushing at semantic boundaries.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_to_writer_streaming, Options};
///
/// let broken = r#"{a:1, items: [1, 2, 3]}"#;
/// let mut output = Vec::new();
/// repair_to_writer_streaming(broken, &Options::default(), &mut output)?;
/// assert!(output.len() > 0);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_to_writer_streaming<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    repair::repair_to_writer_streaming(input, opts, writer)
}

// ============================================================================
// Streaming Chunks API
// ============================================================================

/// Repair a sequence of UTF-8 chunks using the streaming engine and collect into a String.
///
/// If `opts.stream_ndjson_aggregate` is true, returns a single JSON array;
/// otherwise concatenates outputs.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_chunks_to_string, Options};
///
/// let chunks = vec!["{a:", "1", "}"];
/// let repaired = repair_chunks_to_string(chunks, &Options::default())?;
/// assert_eq!(repaired, r#"{"a":1}"#);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_chunks_to_string<'a, I>(chunks: I, opts: &Options) -> Result<String, RepairError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut r = StreamRepairer::new(opts.clone());
    let mut out = String::new();
    for c in chunks.into_iter() {
        if let Some(s) = r.push(c)? {
            out.push_str(&s);
        }
    }
    if let Some(tail) = r.flush()? {
        out.push_str(&tail);
    }
    Ok(out)
}

/// Repair a sequence of UTF-8 chunks using the streaming engine and write into `writer`.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_chunks_to_writer, Options};
///
/// let chunks = vec!["{a:", "1", "}"];
/// let mut output = Vec::new();
/// repair_chunks_to_writer(chunks, &Options::default(), &mut output)?;
/// assert_eq!(output, br#"{"a":1}"#);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_chunks_to_writer<'a, I, W>(
    chunks: I,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError>
where
    I: IntoIterator<Item = &'a str>,
    W: Write,
{
    let mut r = StreamRepairer::new(opts.clone());
    for c in chunks.into_iter() {
        r.push_to_writer(c, writer)?;
    }
    r.flush_to_writer(writer)
}

// ============================================================================
// Parse to Value API (requires serde feature)
// ============================================================================

#[cfg(feature = "serde")]
/// Repair and then parse into `serde_json::Value`.
///
/// This is a convenience function that combines repair and parsing.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_to_value, Options};
///
/// let broken = r#"{name: 'John', age: 30}"#;
/// let value = repair_to_value(broken, &Options::default())?;
/// assert_eq!(value["name"], "John");
/// assert_eq!(value["age"], 30);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_to_value(input: &str, opts: &Options) -> Result<serde_json::Value, RepairError> {
    let s = repair_to_string(input, opts)?;
    let v = serde_json::from_str(&s).map_err(|e| RepairError::from_serde("parse", e))?;
    Ok(v)
}

#[cfg(feature = "serde")]
/// Alias for [`repair_to_value`] - repairs broken JSON and parses it into a `serde_json::Value`.
///
/// This naming matches the Python `json.loads()` and `json_repair.loads()` convention.
///
/// # Examples
///
/// ```
/// use jsonrepair::{loads, Options};
///
/// let broken = r#"{name: 'John', age: 30}"#;
/// let value = loads(broken, &Options::default())?;
/// assert_eq!(value["name"], "John");
/// assert_eq!(value["age"], 30);
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn loads(input: &str, opts: &Options) -> Result<serde_json::Value, RepairError> {
    repair_to_value(input, opts)
}

// ============================================================================
// File and Reader API (requires serde feature)
// ============================================================================

#[cfg(feature = "serde")]
/// Repair and parse JSON from a reader (e.g., file, network stream).
///
/// This is equivalent to reading all content from the reader and calling [`loads`].
///
/// # Examples
///
/// ```no_run
/// use jsonrepair::{load, Options};
/// use std::fs::File;
///
/// let file = File::open("broken.json")?;
/// let value = load(file, &Options::default())?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn load<R: std::io::Read>(
    mut reader: R,
    opts: &Options,
) -> Result<serde_json::Value, RepairError> {
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .map_err(|e| RepairError::from_serde("read", serde_json::Error::io(e)))?;
    loads(&content, opts)
}

#[cfg(feature = "serde")]
/// Repair and parse JSON from a file path.
///
/// This is a convenience wrapper around [`load`] that opens the file for you.
///
/// # Examples
///
/// ```no_run
/// use jsonrepair::{from_file, Options};
///
/// let value = from_file("broken.json", &Options::default())?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn from_file<P: AsRef<std::path::Path>>(
    path: P,
    opts: &Options,
) -> Result<serde_json::Value, RepairError> {
    let file = std::fs::File::open(path)
        .map_err(|e| RepairError::from_serde("open file", serde_json::Error::io(e)))?;
    load(file, opts)
}

// ============================================================================
// Logging API
// ============================================================================

/// Repair a potentially invalid JSON string and return both the string result and a repair log.
///
/// This is useful for debugging or understanding what repairs were made.
///
/// # Examples
///
/// ```
/// use jsonrepair::{repair_to_string_with_log, Options};
///
/// let mut opts = Options::default();
/// opts.log_context_window = 12;
///
/// let (repaired, log) = repair_to_string_with_log("[1, 2 /*c*/, 3]", &opts)?;
/// assert_eq!(repaired, "[1,2,3]");
/// // Note: when built without the `logging` feature, `log` may be empty.
/// // With `logging` enabled, `log` will contain entries describing the fixes.
/// # Ok::<(), jsonrepair::RepairError>(())
/// ```
pub fn repair_to_string_with_log(
    input: &str,
    opts: &Options,
) -> Result<(String, Vec<RepairLogEntry>), RepairError> {
    repair::repair_to_string_with_log(input, opts)
}

#[cfg(test)]
mod tests;
