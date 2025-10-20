mod classify;
pub mod cli;
pub mod error;
pub mod options;
mod repair;
pub mod stream;
mod emit;
mod parser;

pub use error::{RepairError, RepairErrorKind};
pub use options::{LeadingZeroPolicy, Options};
pub use repair::RepairLogEntry;
pub use stream::StreamRepairer;

/// Repair a potentially invalid JSON string into a valid JSON string.
/// This function focuses on common issues like unquoted keys/strings,
/// missing commas/colons, comments, and unclosed brackets/braces.
pub fn repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError> {
    repair::repair_to_string(input, opts)
}

use std::io::Write;

/// Repair a potentially invalid JSON string and write the result into an `io::Write`.
/// This avoids an extra copy of the final string when the caller intends to stream to a sink.
pub fn repair_to_writer<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    let s = repair::repair_to_string(input, opts)?;
    writer
        .write_all(s.as_bytes())
        .map_err(|e| RepairError::from_serde("write", serde_json::Error::io(e)))
}

/// Convenience: repair a sequence of UTF-8 chunks using the streaming engine and collect into a String.
/// If `opts.stream_ndjson_aggregate` is true, returns a single JSON array; otherwise concatenates outputs.
pub fn repair_chunks_to_string<'a, I>(chunks: I, opts: &Options) -> Result<String, RepairError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut r = StreamRepairer::new(opts.clone());
    let mut out = String::new();
    for c in chunks.into_iter() {
        let s = r.push(c)?;
        if !s.is_empty() {
            out.push_str(&s);
        }
    }
    let tail = r.flush()?;
    if !tail.is_empty() {
        out.push_str(&tail);
    }
    Ok(out)
}

/// Convenience: repair a sequence of UTF-8 chunks using the streaming engine and write into `writer`.
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

/// Repair a potentially invalid JSON string and stream the output into a writer while parsing.
/// This reduces peak memory usage for very large inputs by flushing at semantic boundaries.
pub fn repair_to_writer_streaming<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    repair::repair_to_writer_streaming(input, opts, writer)
}

#[cfg(feature = "serde")]
/// Repair and then parse into `serde_json::Value`.
pub fn repair_to_value(input: &str, opts: &Options) -> Result<serde_json::Value, RepairError> {
    let s = repair_to_string(input, opts)?;
    let v = serde_json::from_str(&s).map_err(|e| RepairError::from_serde("parse", e))?;
    Ok(v)
}

/// Repair a potentially invalid JSON string and return both the string result and a repair log.
pub fn repair_to_string_with_log(
    input: &str,
    opts: &Options,
) -> Result<(String, Vec<RepairLogEntry>), RepairError> {
    repair::repair_to_string_with_log(input, opts)
}

#[cfg(test)]
mod tests;
