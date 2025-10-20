#![allow(clippy::collapsible_if)]
#![allow(clippy::needless_lifetimes)]

use crate::emit::{Emitter, JRResult, StringEmitter, WriterEmitter};
use crate::error::{RepairError, RepairErrorKind};
use crate::repair::RepairLogEntry;
use crate::options::Options;
// Hand-written recursive descent parser using &str slicing for zero-copy parsing

pub(crate) mod lex;
mod strings;
mod number;
mod object;
mod array;

use lex::{skip_bom, skip_ws_and_comments, starts_with_ident, take_ident, take_symbol_until_delim};
use strings::{emit_json_string_from_lit, parse_string_literal_concat_fast};
use number::parse_number_token;
use object::parse_object;
use array::parse_array;

fn to_err(pos: usize, msg: impl Into<String>) -> RepairError {
    RepairError::new(RepairErrorKind::Parse(msg.into()), pos)
}

#[derive(Default)]
pub(crate) struct Logger {
    enable: bool,
    track_path: bool,
    entries: Vec<RepairLogEntry>,
    path: Vec<PathElem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathElem {
    Index(usize),
    Key(String),
}

impl Logger {
    pub(crate) fn new(enable: bool, track_path: bool) -> Self {
        Self { enable, track_path, entries: Vec::new(), path: Vec::new() }
    }
    fn log(&mut self, message: &'static str) {
        if !self.enable { return; }
        let path = if self.track_path {
            Some(self.format_path())
        } else { None };
        self.entries.push(RepairLogEntry{ position: 0, message, context: String::new(), path });
    }
    fn format_path(&self) -> String {
        let mut s = String::from("$");
        for el in &self.path {
            match el {
                PathElem::Index(i) => {
                    s.push('[');
                    s.push_str(&i.to_string());
                    s.push(']');
                }
                PathElem::Key(k) => {
                    s.push('[');
                    s.push('"');
                    for ch in k.chars() {
                        match ch {
                            '"' => s.push_str("\\\""),
                            '\\' => s.push_str("\\\\"),
                            _ => s.push(ch),
                        }
                    }
                    s.push('"');
                    s.push(']');
                }
            }
        }
        s
    }
    fn push_key(&mut self, k: String) { if self.track_path { self.path.push(PathElem::Key(k)); } }
    fn pop_key(&mut self) { if self.track_path { let _ = self.path.pop(); } }
    fn push_index(&mut self, i: usize) { if self.track_path { self.path.push(PathElem::Index(i)); } }
    fn pop_index(&mut self) { if self.track_path { let _ = self.path.pop(); } }
    pub(crate) fn into_entries(self) -> Vec<RepairLogEntry> { self.entries }
}

pub(crate) fn repair_to_string_impl(input: &str, opts: &Options) -> Result<String, RepairError> {
    let mut s = pre_trim_wrappers(input, opts);

    // Fast path: if input is already valid JSON, short-circuit
    #[cfg(feature = "serde")]
    {
        if !opts.ensure_ascii && opts.assume_valid_json_fastpath {
            // Skip full validation for maximum speed when explicitly allowed.
            return Ok(s.to_string());
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
            if !opts.ensure_ascii {
                return Ok(s.to_string());
            } else {
                use serde::Serialize;
                let mut buf: Vec<u8> = Vec::with_capacity(s.len());
                let mut ser = serde_json::Serializer::with_formatter(&mut buf, AsciiEscaper);
                val.serialize(&mut ser).map_err(|e| to_err(0, format!("serde serialize error: {}", e)))?;
                let out = String::from_utf8(buf).map_err(|e| to_err(0, format!("utf8 error: {}", e)))?;
                return Ok(out);
            }
        }
    }

    let mut logger = Logger{ enable: false, track_path: false, entries: Vec::new(), path: Vec::new() };
    let out = parse_root_many_string_fast(&mut s, opts, &mut logger)?;
    Ok(out)
}

pub(crate) fn repair_to_writer_impl<W: std::io::Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    let mut s = pre_trim_wrappers(input, opts);

    // Fast path when input is already valid JSON.
    #[cfg(feature = "serde")]
    {
        use serde::Serialize;
        if !opts.ensure_ascii && opts.assume_valid_json_fastpath {
            writer
                .write_all(s.as_bytes())
                .map_err(|e| to_err(0, format!("io write error: {}", e)))?;
            return Ok(());
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
            if !opts.ensure_ascii {
                writer
                    .write_all(s.as_bytes())
                    .map_err(|e| to_err(0, format!("io write error: {}", e)))?;
                return Ok(());
            } else {
                let mut ser = serde_json::Serializer::with_formatter(writer, AsciiEscaper);
                val.serialize(&mut ser)
                    .map_err(|e| to_err(0, format!("serde serialize error: {}", e)))?;
                return Ok(());
            }
        }
    }

    let mut emitter = WriterEmitter::with_capacity(writer, s.len().saturating_add(8));
    let mut logger = Logger{ enable: false, track_path: false, entries: Vec::new(), path: Vec::new() };
    parse_root_many(&mut s, opts, &mut emitter, &mut logger)?;
    emitter.flush_all()
}

pub(crate) fn pre_trim_wrappers<'i>(input: &'i str, opts: &Options) -> &'i str {
    let mut s = input;
    // BOM
    skip_bom(&mut s);
    // Markdown fence: ```lang\n ... ```
    if opts.fenced_code_blocks && let Some(body) = trim_fenced_block(s) {
        s = body;
    }
    // JSONP: ident( ... ) ;
    // 可嵌套，多层剥离
    while let Some(inner) = trim_jsonp(s) {
        s = inner;
    }
    s
}

fn trim_fenced_block(s: &str) -> Option<&str> {
    // Accept ``` optionally followed by language tokens (letters/digits/underscore, case-insensitive),
    // optional whitespace and newline (\n or \r\n). Capture until the matching closing ```.
    if !s.starts_with("```") { return None; }
    let mut idx = 3;
    // optional language
    while idx < s.len() {
        let ch = s[idx..].chars().next().unwrap();
        if ch.is_ascii_alphanumeric() || ch == '_' { idx += ch.len_utf8(); } else { break; }
    }
    // optional spaces/tabs
    while idx < s.len() {
        let ch = s[idx..].chars().next().unwrap();
        if ch == ' ' || ch == '\t' { idx += ch.len_utf8(); } else { break; }
    }
    // optional newline (\n or \r\n)
    if s[idx..].starts_with("\r\n") { idx += 2; }
    else if s[idx..].starts_with("\n") { idx += 1; }
    let body = &s[idx..];
    if let Some(end) = body.find("```") {
        Some(&body[..end])
    } else {
        None
    }
}

fn trim_jsonp(s: &str) -> Option<&str> {
    // naive jsonp: <ident>\s* ( ... ) [;]  -> return inner
    let rest = s.trim_start();
    if !starts_with_ident(rest) {
        return None;
    }
    let (_name, after) = take_ident(rest);
    let after = after.trim_start();
    if !after.starts_with('(') {
        return None;
    }
    // find last ')' and take inner; naive but works for typical JSON
    if let Some(idx) = after.rfind(')') {
        let inner = &after[1..idx];
        return Some(inner);
    }
    None
}

pub(crate) fn parse_root_many<'i, E: Emitter>(input: &mut &'i str, opts: &Options, out: &mut E, logger: &mut Logger) -> JRResult<()> {
    skip_ws_and_comments(input, opts);
    if input.is_empty() {
        return Ok(());
    }

    // Parse first value into a small buffer so we can decide whether to aggregate into an array
    let mut first = String::new();
    {
        let mut se = StringEmitter::new(&mut first);
        parse_value(input, opts, &mut se, logger)?;
    }

    // After first value, skip optional WS/comments and a single comma
    skip_ws_and_comments(input, opts);
    if input.starts_with(',') {
        *input = &input[1..];
        skip_ws_and_comments(input, opts);
    }

    let has_more = starts_value(input);
    if has_more {
        out.emit_char('[')?;
        out.emit_str(&first)?;
        while !input.is_empty() {
            skip_ws_and_comments(input, opts);
            if input.is_empty() { break; }
            if input.starts_with(']') || input.starts_with('}') { break; }
            if !starts_value(input) { break; }
            out.emit_char(',')?;
            parse_value(input, opts, out, logger)?;
            skip_ws_and_comments(input, opts);
            if input.starts_with(',') {
                *input = &input[1..];
            }
        }
        out.emit_char(']')?;
    } else {
        out.emit_str(&first)?;
    }

    // best-effort: drop trailing wrappers like ");" if present
    skip_ws_and_comments(input, opts);
    if input.starts_with(')') { *input = &input[1..]; }
    if input.starts_with(';') { *input = &input[1..]; }
    Ok(())
}

/// Optimized root parser for String output: avoid buffering the first value separately when
/// there's only a single root value (the common case). Falls back to array aggregation when
/// multiple root values are detected.
fn parse_root_many_string_fast<'i>(input: &mut &'i str, opts: &Options, logger: &mut Logger) -> JRResult<String> {
    let mut out = String::new();
    let mut se = StringEmitter::new(&mut out);
    skip_ws_and_comments(input, opts);
    if input.is_empty() {
        return Ok(out);
    }
    // Parse first value directly into out
    parse_value(input, opts, &mut se, logger)?;

    // Probe if there are more values (optional comma + value start)
    skip_ws_and_comments(input, opts);
    if input.starts_with(',') {
        *input = &input[1..];
        skip_ws_and_comments(input, opts);
    }
    let has_more = starts_value(input);
    if !has_more {
        // best-effort: drop trailing wrappers like ") ;" if present
        skip_ws_and_comments(input, opts);
        if input.starts_with(')') { *input = &input[1..]; }
        if input.starts_with(';') { *input = &input[1..]; }
        return Ok(out);
    }

    // 🔧 NDJSON stream fallback disabled - direct aggregation is faster for benchmarks
    // The streaming processor has overhead that makes it slower for small NDJSON inputs

    // Multiple values: aggregate into array
    let mut agg = String::with_capacity(out.len().saturating_add(8));
    agg.push('[');
    agg.push_str(&out);
    let mut agg_se = StringEmitter::new(&mut agg);
    while !input.is_empty() {
        skip_ws_and_comments(input, opts);
        if input.is_empty() { break; }
        if input.starts_with(']') || input.starts_with('}') { break; }
        if !starts_value(input) { break; }
        agg_se.emit_char(',')?;
        parse_value(input, opts, &mut agg_se, logger)?;
        skip_ws_and_comments(input, opts);
        if input.starts_with(',') { *input = &input[1..]; }
    }
    agg_se.emit_char(']')?;
    // best-effort: drop trailing JSONP artifacts
    skip_ws_and_comments(input, opts);
    if input.starts_with(')') { *input = &input[1..]; }
    if input.starts_with(';') { *input = &input[1..]; }
    Ok(agg)
}

fn starts_value(s: &str) -> bool {
    let s = s.trim_start();
    match s.chars().next() {
        Some('{') | Some('[') | Some('"') | Some('\'') | Some('-') => true,
        Some(c) if c.is_ascii_digit() => true,
        Some(c) if c.is_ascii_alphabetic() => true,
        _ => false,
    }
}

fn parse_value<'i, E: Emitter>(input: &mut &'i str, opts: &Options, out: &mut E, logger: &mut Logger) -> JRResult<()> {
    skip_ws_and_comments(input, opts);
    if input.is_empty() {
        return Err(to_err(0, "unexpected end while parsing value"));
    }
    let c = input.chars().next().unwrap();
    match c {
        '{' => parse_object(input, opts, out, logger),
        '[' => parse_array(input, opts, out, logger),
        '"' | '\'' => parse_string_literal_concat_fast(input, opts, out),
        '/' => parse_regex_literal(input, opts, out),
        '-' => {
            // Special-case JS non-finite: -Infinity
            if opts.normalize_js_nonfinite && input.starts_with("-Infinity") {
                *input = &input[9..];
                out.emit_str("null")
            } else {
                parse_number_token(input, opts, out)
            }
        }
        c if c == '.' || c.is_ascii_digit() => parse_number_token(input, opts, out),
        _ => parse_symbol_or_unquoted_string(input, opts, out, logger),
    }
}

pub(crate) fn parse_symbol_or_unquoted_string<'i, E: Emitter>(input: &mut &'i str, opts: &Options, out: &mut E, logger: &mut Logger) -> JRResult<()> {
    let s = *input;
    let (tok, rest) = take_ident(s);
    if !tok.is_empty() {
        *input = rest;
        return match tok {
            "true" => out.emit_str("true"),
            "false" => out.emit_str("false"),
            "null" => out.emit_str("null"),
            // pythonic
            "True" if opts.allow_python_keywords => { logger.log("normalized python keyword"); out.emit_str("true") },
            "False" if opts.allow_python_keywords => { logger.log("normalized python keyword"); out.emit_str("false") },
            "None" if opts.allow_python_keywords => { logger.log("normalized python keyword"); out.emit_str("null") },
            // js non-finite
            "NaN" | "Infinity" | "-Infinity" if opts.normalize_js_nonfinite => out.emit_str("null"),
            // undefined
            "undefined" if opts.repair_undefined => { logger.log("replaced undefined with null"); out.emit_str("null") },
            _ => emit_json_string_from_lit(out, tok, opts.ensure_ascii),
        };
    }
    // Non-ASCII (e.g., Chinese) or punctuation-start symbols: take a run until delimiters and quote it.
    let sym = take_symbol_until_delim(input);
    if sym.is_empty() {
        // fallback: quote single char if any
        if !s.is_empty() {
            let ch = s.chars().next().unwrap();
            *input = &s[ch.len_utf8()..];
            return emit_json_string_from_lit(out, ch.encode_utf8(&mut [0;4]), opts.ensure_ascii);
        }
        return Ok(());
    }
    emit_json_string_from_lit(out, sym, opts.ensure_ascii)
}

fn parse_regex_literal<'i, E: Emitter>(input: &mut &'i str, _opts: &Options, out: &mut E) -> JRResult<()> {
    // 解析 /.../flags 成 JSON 字符串，尽量保留原样（包含斜杠和 flags）
    let s = *input;
    if !s.starts_with('/') { return emit_json_string_from_lit(out, "/", false); }
    let mut i = 1usize; // after first '/'
    let mut esc = false;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        let l = ch.len_utf8();
        i += l;
        if esc { esc = false; continue; }
        if ch == '\\' { esc = true; continue; }
        if ch == '/' {
            // capture flags
            let mut j = i;
            while j < s.len() {
                let ch2 = s[j..].chars().next().unwrap();
                if ch2.is_ascii_alphabetic() { j += ch2.len_utf8(); } else { break; }
            }
            // Build a cleaned representation: remove escapes for forward slash in the body
            let lit = &s[..j]; // includes both slashes and flags
            let mut cleaned = String::with_capacity(lit.len());
            // split into /body/ and optional flags
            let body = &lit[1..i-1]; // between the two '/'
            let flags = &lit[i..j];
            cleaned.push('/');
            let mut k = 0usize;
            while k < body.len() {
                let ch = body[k..].chars().next().unwrap();
                let l = ch.len_utf8();
                if ch == '\\' {
                    // if escaping a forward slash, drop the backslash
                    if k + l < body.len() && body[k + l..].starts_with('/') {
                        cleaned.push('/');
                        k += l + '/'.len_utf8();
                        continue;
                    }
                    // keep the backslash for other escapes
                    cleaned.push('\\');
                    k += l;
                    continue;
                }
                cleaned.push(ch);
                k += l;
            }
            cleaned.push('/');
            cleaned.push_str(flags);
            *input = &s[j..];
            return emit_json_string_from_lit(out, &cleaned, false);
        }
    }
    // 未闭合，回退为到结尾的文本
    let lit = s;
    *input = &s[s.len()..];
    emit_json_string_from_lit(out, lit, false)
}

// ASCII-only string formatter for serde_json serializer
// Ensures all non-ASCII characters are escaped as \uXXXX (and surrogate pairs when needed).
#[cfg(feature = "serde")]
struct AsciiEscaper;

#[cfg(feature = "serde")]
impl serde_json::ser::Formatter for AsciiEscaper {
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        let mut start = 0usize;
        let fragment_bytes = fragment.as_bytes();
        for (i, ch) in fragment.char_indices() {
            if ch <= '\u{7F}' { continue; }
            if i > start {
                writer.write_all(&fragment_bytes[start..i])?;
            }
            let cp = ch as u32;
            if cp <= 0xFFFF {
                // Safe: char in Rust is not a surrogate half
                write!(writer, "\\u{:04X}", cp)?;
            } else {
                // Encode as surrogate pair
                let v = cp - 0x10000;
                let high = 0xD800 + ((v >> 10) & 0x3FF);
                let low = 0xDC00 + (v & 0x3FF);
                write!(writer, "\\u{:04X}\\u{:04X}", high, low)?;
            }
            start = i + ch.len_utf8();
        }
        if start < fragment.len() {
            writer.write_all(&fragment_bytes[start..])?;
        }
        Ok(())
    }
}

