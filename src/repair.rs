use crate::classify::*;
use crate::error::{RepairError, RepairErrorKind};
use crate::options::{LeadingZeroPolicy, Options};
use memchr::{memchr, memchr2, memchr3};
use std::io::Write;
// Generic output abstraction for non-streaming writer-friendly paths.
pub trait Out {
    fn push_char(&mut self, c: char);
    fn push_str(&mut self, s: &str);
    fn last_char(&self) -> Option<char>;
    fn pop_last(&mut self) -> Option<char>;
    fn flush_hint(&mut self) {}
}

impl Out for String {
    #[inline]
    fn push_char(&mut self, c: char) {
        self.push(c);
    }
    #[inline]
    fn push_str(&mut self, s: &str) {
        self.push_str(s);
    }
    #[inline]
    fn last_char(&self) -> Option<char> {
        self.chars().next_back()
    }
    #[inline]
    fn pop_last(&mut self) -> Option<char> {
        self.pop()
    }
    #[inline]
    fn flush_hint(&mut self) {}
}

struct WriterOut<'a, W: Write> {
    w: &'a mut W,
    tail: String,
}

impl<'a, W: Write> WriterOut<'a, W> {
    fn new(w: &'a mut W) -> Self {
        Self {
            w,
            tail: String::with_capacity(8192),
        }
    }
    fn flush_chunk(&mut self) {
        const KEEP: usize = 64;
        if self.tail.len() > KEEP {
            let split_at = self.tail.len() - KEEP;
            let prefix = &self.tail.as_bytes()[..split_at];
            let _ = self.w.write_all(prefix);
            self.tail.drain(..split_at);
        }
    }
    fn finish(&mut self) -> std::io::Result<()> {
        if !self.tail.is_empty() {
            self.w.write_all(self.tail.as_bytes())?;
            self.tail.clear();
        }
        Ok(())
    }
}

impl<'a, W: Write> Out for WriterOut<'a, W> {
    fn push_char(&mut self, c: char) {
        self.tail.push(c);
    }
    fn push_str(&mut self, s: &str) {
        self.tail.push_str(s);
    }
    fn last_char(&self) -> Option<char> {
        self.tail.chars().next_back()
    }
    fn pop_last(&mut self) -> Option<char> {
        self.tail.pop()
    }
    fn flush_hint(&mut self) {
        self.flush_chunk();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairLogEntry {
    pub position: usize,
    pub message: &'static str,
    pub context: String,
    pub path: Option<String>,
}

#[derive(Default)]
struct Logger {
    enable: bool,
    entries: Vec<RepairLogEntry>,
    track_path: bool,
    path: Vec<PathElem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathElem {
    Index(usize),
    Key(String),
}

impl Logger {
    #[inline]
    fn log(&mut self, position: usize, message: &'static str) {
        if self.enable {
            let path = if self.track_path {
                Some(self.format_path())
            } else {
                None
            };
            self.entries.push(RepairLogEntry {
                position,
                message,
                context: String::new(),
                path,
            });
        }
    }

    #[inline]
    fn log_with_context(&mut self, position: usize, message: &'static str, context: String) {
        if self.enable {
            let path = if self.track_path {
                Some(self.format_path())
            } else {
                None
            };
            self.entries.push(RepairLogEntry {
                position,
                message,
                context,
                path,
            });
        }
    }

    #[inline]
    fn push_index(&mut self, idx: usize) {
        if self.track_path {
            self.path.push(PathElem::Index(idx));
        }
    }
    #[inline]
    fn pop(&mut self) {
        if self.track_path {
            let _ = self.path.pop();
        }
    }
    #[inline]
    fn push_key_literal(&mut self, key_json_literal: &str) {
        if self.track_path {
            self.path.push(PathElem::Key(key_json_literal.to_string()));
        }
    }
    #[inline]
    fn format_path(&self) -> String {
        let mut s = String::from("$");
        for elem in &self.path {
            match elem {
                PathElem::Index(i) => {
                    s.push('[');
                    s.push_str(&i.to_string());
                    s.push(']');
                }
                PathElem::Key(k) => {
                    s.push('[');
                    s.push_str(k);
                    s.push(']');
                }
            }
        }
        s
    }
}

#[inline]
fn build_context(chars: &[char], pos: usize, win_usize: usize) -> String {
    let win: isize = win_usize as isize;
    let len = chars.len() as isize;
    let p = pos as isize;
    let start = (p - win).max(0) as usize;
    let end = (p + win).min(len) as usize;
    chars[start..end].iter().collect::<String>()
}

#[inline]
fn byte_to_char_index(ascii: bool, offs: &[usize], target: usize, char_len: usize) -> usize {
    if ascii {
        // In pure-ASCII inputs, byte offsets equal char indices.
        return target.min(char_len);
    }
    match offs.binary_search(&target) {
        Ok(idx) => idx.min(char_len),
        Err(idx) => idx.min(char_len),
    }
}

#[inline]
fn skip_bom(chars: &[char], i: &mut usize) {
    while let Some('\u{FEFF}') = chars.get(*i) {
        *i += 1;
    }
}

#[inline]
fn earliest_of(bytes: &[u8], needles: &[u8]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for &n in needles {
        if let Some(p) = memchr(n, bytes) {
            best = Some(best.map_or(p, |b| p.min(b)));
        }
    }
    best
}

#[inline]
fn fast_forward_spaces_to_next_delim(
    i: &mut usize,
    chars: &[char],
    bytes: &[u8],
    offs: &[usize],
    candidates: &[u8],
    ascii: bool,
) -> bool {
    if let Some(&c0) = chars.get(*i)
        && (c0 == ' ' || c0 == '\t' || c0 == '\n' || c0 == '\r')
        && let Some(start_b) = offs.get(*i).copied()
        && let Some(rel) = earliest_of(bytes.get(start_b..).unwrap_or(&[]), candidates)
    {
        let slice = &bytes[start_b..start_b + rel];
        if slice
            .iter()
            .all(|&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r')
        {
            *i = byte_to_char_index(ascii, offs, start_b + rel, chars.len());
            return true;
        }
    }
    false
}

#[inline]
fn fast_forward_spaces_to_char(
    i: &mut usize,
    chars: &[char],
    bytes: &[u8],
    offs: &[usize],
    target: u8,
    ascii: bool,
) -> bool {
    if let Some(&c0) = chars.get(*i)
        && (c0 == ' ' || c0 == '\t' || c0 == '\n' || c0 == '\r')
        && let Some(start_b) = offs.get(*i).copied()
        && let Some(rel) = memchr(target, bytes.get(start_b..).unwrap_or(&[]))
    {
        let slice = &bytes[start_b..start_b + rel];
        if slice
            .iter()
            .all(|&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r')
        {
            *i = byte_to_char_index(ascii, offs, start_b + rel, chars.len());
            return true;
        }
    }
    false
}

#[inline]
fn find_next_unquoted_break(bytes: &[u8], include_colon: bool) -> Option<usize> {
    // Search for the earliest occurrence of any delimiter that should terminate an unquoted symbol.
    // Delimiters: ',', '[', ']', '{', '}', '\n', '\r', '(', ')', ':' (when key), '"', '\''.
    // Note: we deliberately do NOT include '/' to keep behavior consistent with the char loop.
    let mut best: Option<usize> = None;
    let mut upd = |p: Option<usize>| {
        if let Some(x) = p {
            match best {
                Some(b) => {
                    if x < b {
                        best = Some(x);
                    }
                }
                None => best = Some(x),
            }
        }
    };
    upd(memchr3(b',', b'[', b']', bytes));
    upd(memchr3(b'{', b'}', b'\n', bytes));
    upd(memchr3(b'\r', b'(', b')', bytes));
    upd(memchr2(b'"', b'\'', bytes));
    if include_colon {
        upd(memchr(b':', bytes));
    }
    best
}

fn is_suspect_numeric_token(tok: &str, opts: &Options) -> bool {
    if !opts.number_quote_suspicious {
        return false;
    }
    let bytes = tok.as_bytes();
    // Quick allow: pure number grammar with tolerances
    // Reject if contains any alpha except e/E
    if bytes
        .iter()
        .any(|&b| (b as char).is_alphabetic() && b != b'e' && b != b'E')
    {
        return true;
    }
    // Reject if contains '/'
    if bytes.contains(&b'/') {
        return true;
    }
    // Count dots
    let dot_count = bytes.iter().filter(|&&b| b == b'.').count();
    if dot_count > 1 {
        return true;
    }
    // Hyphen positions: allow leading '-', or right after e/E; others -> suspicious
    for (idx, &b) in bytes.iter().enumerate() {
        if b == b'-' {
            if idx == 0 {
                continue;
            }
            let prev = bytes.get(idx.wrapping_sub(1)).copied().unwrap_or(b' ');
            if prev != b'e' && prev != b'E' {
                return true;
            }
        }
    }
    false
}

pub(crate) fn repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError> {
    let mut out = String::with_capacity(input.len() + 8);
    let chars: Vec<char> = input.chars().collect();
    let bytes = input.as_bytes();
    let ascii = input.is_ascii();
    let mut char_offsets: Vec<usize> = Vec::with_capacity(chars.len());
    for (bi, _c) in input.char_indices() {
        char_offsets.push(bi);
    }
    let mut i: usize = 0;
    let mut logger = Logger {
        enable: false,
        entries: Vec::new(),
        track_path: false,
        path: Vec::new(),
    };

    // Skip BOM and then fenced code block like ```json ... ``` at the beginning
    skip_bom(&chars, &mut i);
    let mut content_start = i;
    if opts.fenced_code_blocks && skip_markdown_fence_start(&mut i, &chars) {
        content_start = i;
    }

    // Parse the first value
    parse_value(
        &mut i,
        &chars,
        &mut out,
        opts,
        &mut logger,
        bytes,
        &char_offsets,
        ascii,
    )?;
    parse_ws_and_comments(
        &mut i,
        &chars,
        &mut out,
        opts,
        &mut logger,
        bytes,
        &char_offsets,
        ascii,
    );

    // Try to detect NDJSON: root-level sequential values
    if matches!(chars.get(i), Some(',')) {
        i += 1;
        parse_ws_and_comments(
            &mut i,
            &chars,
            &mut out,
            opts,
            &mut logger,
            bytes,
            &char_offsets,
            ascii,
        );
    }
    let is_next_value = chars
        .get(i)
        .copied()
        .map(is_start_of_value)
        .unwrap_or(false);
    if is_next_value {
        // Rebuild output as an array of values from the beginning without emitting whitespace between values
        let mut nd_out = String::with_capacity(out.len() + 8);
        nd_out.push('[');

        // Helper to parse one element while suppressing inter-element whitespace/comment emission
        let mut j = content_start;
        let mut first = true;
        while j < chars.len() {
            // skip whitespace/comments without emitting
            parse_ws_and_comments_silent(&mut j, &chars, opts, bytes, &char_offsets, ascii);
            if j >= chars.len() {
                break;
            }
            // Stop if we encounter redundant trailing closers
            if matches!(chars.get(j), Some('}' | ']')) {
                j += 1;
                continue;
            }
            // If not start of a value, abort loop
            if !chars
                .get(j)
                .copied()
                .map(is_start_of_value)
                .unwrap_or(false)
            {
                break;
            }
            if !first {
                nd_out.push(',');
            }
            first = false;

            // capture element string by parsing into temp buffer
            let mut elem = String::new();
            parse_value(
                &mut j,
                &chars,
                &mut elem,
                opts,
                &mut logger,
                bytes,
                &char_offsets,
                ascii,
            )?;
            nd_out.push_str(&elem);
            // consume possible comma between NDJSON values
            parse_ws_and_comments_silent(&mut j, &chars, opts, bytes, &char_offsets, ascii);
            if matches!(chars.get(j), Some(',')) {
                j += 1;
            }
        }
        nd_out.push(']');
        // advance i to where j ended so we can skip trailing fence
        i = j;
        if opts.fenced_code_blocks {
            skip_markdown_fence_end(&mut i, &chars);
        }
        return Ok(nd_out);
    }

    // Skip redundant end braces/brackets
    while let Some(c) = chars.get(i) {
        if matches!(c, '}' | ']') {
            i += 1;
            parse_ws_and_comments(
                &mut i,
                &chars,
                &mut out,
                opts,
                &mut logger,
                bytes,
                &char_offsets,
                ascii,
            );
        } else {
            break;
        }
    }
    // Append trailing whitespace only
    while let Some(&c) = chars.get(i) {
        if is_whitespace(c) {
            out.push(c);
            i += 1;
        } else {
            break;
        }
    }
    // Skip trailing fenced block if present
    if opts.fenced_code_blocks {
        skip_markdown_fence_end(&mut i, &chars);
    }
    Ok(out)
}

pub(crate) fn repair_to_writer_streaming<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    let mut out = WriterOut::new(writer);
    let chars: Vec<char> = input.chars().collect();
    let bytes = input.as_bytes();
    let ascii = input.is_ascii();
    let mut char_offsets: Vec<usize> = Vec::with_capacity(chars.len());
    for (bi, _c) in input.char_indices() {
        char_offsets.push(bi);
    }
    let mut i: usize = 0;
    let mut logger = Logger {
        enable: false,
        entries: Vec::new(),
        track_path: false,
        path: Vec::new(),
    };

    skip_bom(&chars, &mut i);
    let mut content_start = i;
    if opts.fenced_code_blocks && skip_markdown_fence_start(&mut i, &chars) {
        content_start = i;
    }

    parse_value(
        &mut i,
        &chars,
        &mut out,
        opts,
        &mut logger,
        bytes,
        &char_offsets,
        ascii,
    )?;
    parse_ws_and_comments(
        &mut i,
        &chars,
        &mut out,
        opts,
        &mut logger,
        bytes,
        &char_offsets,
        ascii,
    );

    if matches!(chars.get(i), Some(',')) {
        i += 1;
        parse_ws_and_comments(
            &mut i,
            &chars,
            &mut out,
            opts,
            &mut logger,
            bytes,
            &char_offsets,
            ascii,
        );
    }
    let is_next_value = chars
        .get(i)
        .copied()
        .map(is_start_of_value)
        .unwrap_or(false);
    if is_next_value {
        // Rebuild as array without emitting whitespace between values
        let mut nd_head_written = false;
        let mut j = content_start;
        while j < chars.len() {
            parse_ws_and_comments_silent(&mut j, &chars, opts, bytes, &char_offsets, ascii);
            if j >= chars.len() {
                break;
            }
            if matches!(chars.get(j), Some('}' | ']')) {
                j += 1;
                continue;
            }
            if !chars
                .get(j)
                .copied()
                .map(is_start_of_value)
                .unwrap_or(false)
            {
                break;
            }
            if !nd_head_written {
                out.push_char('[');
                nd_head_written = true;
            } else {
                out.push_char(',');
            }
            parse_value(
                &mut j,
                &chars,
                &mut out,
                opts,
                &mut logger,
                bytes,
                &char_offsets,
                ascii,
            )?;
            parse_ws_and_comments_silent(&mut j, &chars, opts, bytes, &char_offsets, ascii);
            if matches!(chars.get(j), Some(',')) {
                j += 1;
            }
            out.flush_hint();
        }
        if nd_head_written {
            out.push_char(']');
        }
        i = j;
        if opts.fenced_code_blocks {
            skip_markdown_fence_end(&mut i, &chars);
        }
        out.finish().map_err(|e| {
            RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), i)
        })?;
        return Ok(());
    }

    while let Some(c) = chars.get(i) {
        if matches!(c, '}' | ']') {
            i += 1;
            parse_ws_and_comments(
                &mut i,
                &chars,
                &mut out,
                opts,
                &mut logger,
                bytes,
                &char_offsets,
                ascii,
            );
        } else {
            break;
        }
    }
    while let Some(&c) = chars.get(i) {
        if is_whitespace(c) {
            out.push_char(c);
            i += 1;
        } else {
            break;
        }
    }
    if opts.fenced_code_blocks {
        skip_markdown_fence_end(&mut i, &chars);
    }
    out.finish().map_err(|e| {
        RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), i)
    })?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn parse_value(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    logger: &mut Logger,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) -> Result<bool, RepairError> {
    parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
    if *i >= chars.len() {
        return Err(RepairError::new(RepairErrorKind::UnexpectedEnd, *i));
    }

    // Try object/array first
    if parse_object(i, chars, out, opts, logger, bytes, offs, ascii)? {
        return Ok(true);
    }
    if parse_array(i, chars, out, opts, logger, bytes, offs, ascii)? {
        return Ok(true);
    }

    // String (and concatenation)
    if parse_string_any(i, chars, out, /*is_key*/ false, opts, logger)? {
        parse_concatenated_string(i, chars, out, opts, bytes, offs, ascii)?;
        return Ok(true);
    }

    // Number (skip if suspicious token should be quoted)
    let mut quote_suspect = false;
    if opts.number_quote_suspicious
        && let Some(start_b) = offs.get(*i).copied()
        && let Some(rel) = find_next_unquoted_break(bytes.get(start_b..).unwrap_or(&[]), false)
    {
        let mut end_b = start_b + rel;
        // Trim token at start of comment markers (/* or //) if present before delimiter
        let slice = &bytes[start_b..end_b];
        let mut cut = slice.len();
        let mut idx = 0usize;
        while idx + 1 < slice.len() {
            if slice[idx] == b'/' && (slice[idx + 1] == b'*' || slice[idx + 1] == b'/') {
                cut = idx;
                break;
            }
            idx += 1;
        }
        if cut < slice.len() {
            end_b = start_b + cut;
        }
        let end_i = byte_to_char_index(ascii, offs, end_b, chars.len());
        let tok: String = chars.get(*i..end_i).unwrap_or(&[]).iter().collect();
        let starts_numish = tok.starts_with('-')
            || tok.starts_with('.')
            || tok
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false);
        if starts_numish && is_suspect_numeric_token(&tok, opts) {
            quote_suspect = true;
        }
    }
    if !quote_suspect && parse_number(i, chars, out, opts)? {
        return Ok(true);
    }

    // Keywords
    if parse_keywords(i, chars, out, opts, logger) {
        return Ok(true);
    }

    // JavaScript non-finite numbers: NaN / Infinity / -Infinity -> null (when enabled)
    if parse_js_nonfinite(i, chars, out, opts, logger) {
        return Ok(true);
    }

    // Regex literal
    if parse_regex(i, chars, out, opts)? {
        return Ok(true);
    }

    // Unquoted symbol/string and function call wrappers
    if parse_unquoted(
        i, chars, out, /*is_key*/ false, opts, logger, bytes, offs, ascii,
    ) {
        return Ok(true);
    }

    Ok(false)
}

fn parse_js_nonfinite(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    logger: &mut Logger,
) -> bool {
    if !opts.normalize_js_nonfinite {
        return false;
    }
    let rest: String = chars.get(*i..).unwrap_or(&[]).iter().collect();
    if rest.starts_with("NaN") {
        *i += 3;
        out.push_str("null");
        logger.log_with_context(*i, "normalized NaN to null", String::new());
        return true;
    }
    if rest.starts_with("Infinity") {
        *i += 8;
        out.push_str("null");
        logger.log_with_context(*i, "normalized Infinity to null", String::new());
        return true;
    }
    if rest.starts_with("-Infinity") {
        *i += 9;
        out.push_str("null");
        logger.log_with_context(*i, "normalized -Infinity to null", String::new());
        return true;
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn parse_ws_and_comments(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    logger: &mut Logger,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) {
    loop {
        let start = *i;
        // fast-skip and copy: if a run of spaces/tabs before a newline, copy the whole run at once
        if let Some(&c0) = chars.get(*i)
            && (c0 == ' ' || c0 == '\t')
            && let Some(start_b) = offs.get(*i).copied()
            && let Some(rel) = memchr2(b'\n', b'\r', bytes.get(start_b..).unwrap_or(&[]))
        {
            let line = &bytes[start_b..start_b + rel];
            if line.iter().all(|&b| b == b' ' || b == b'\t') {
                let new_b = start_b + rel; // position at newline, do not consume it here
                let new_i = byte_to_char_index(ascii, offs, new_b, chars.len());
                for &ch in &chars[*i..new_i] {
                    out.push_char(ch);
                }
                *i = new_i;
            }
        }
        while let Some(&c) = chars.get(*i)
            && is_whitespace(c)
        {
            out.push_char(c);
            *i += 1;
        }
        // comments
        if let Some(&'/') = chars.get(*i) {
            if matches!(chars.get(*i + 1), Some('/')) {
                // line comment //...
                // fast skip to end of line using bytes
                let start_b = offs.get(*i).copied().unwrap_or(0);
                let slice = bytes.get(start_b..).unwrap_or(&[]);
                let mut new_b = start_b;
                if slice.len() >= 2 {
                    new_b += 2;
                }
                if let Some(rel) = memchr2(b'\n', b'\r', bytes.get(new_b..).unwrap_or(&[])) {
                    new_b += rel + 1;
                }
                // map byte offset back to char index
                *i = byte_to_char_index(ascii, offs, new_b, chars.len());
                logger.log_with_context(
                    *i,
                    "removed comment",
                    build_context(chars, *i, opts.log_context_window),
                );
                continue;
            }
            if matches!(chars.get(*i + 1), Some('*')) {
                // block comment /* ... */
                let start_b = offs.get(*i).copied().unwrap_or(0) + 2; // skip /*
                let mut scan = start_b;
                while let Some(pos) = memchr(b'*', bytes.get(scan..).unwrap_or(&[])) {
                    let idx = scan + pos;
                    if idx + 1 < bytes.len() && bytes[idx + 1] == b'/' {
                        scan = idx + 2;
                        break;
                    }
                    scan = idx + 1;
                }
                *i = byte_to_char_index(ascii, offs, scan, chars.len());
                logger.log_with_context(
                    *i,
                    "removed comment",
                    build_context(chars, *i, opts.log_context_window),
                );
                continue;
            }
        }
        if opts.tolerate_hash_comments
            && let Some(&'#') = chars.get(*i)
        {
            // line comment #...
            let start_b = offs.get(*i).copied().unwrap_or(0) + 1;
            let mut new_b = start_b;
            if let Some(rel) = memchr2(b'\n', b'\r', bytes.get(start_b..).unwrap_or(&[])) {
                new_b = start_b + rel + 1;
            }
            *i = byte_to_char_index(ascii, offs, new_b, chars.len());
            logger.log_with_context(
                *i,
                "removed comment",
                build_context(chars, *i, opts.log_context_window),
            );
            continue;
        }
        if *i == start {
            break;
        }
    }
}

fn parse_ws_and_comments_silent(
    i: &mut usize,
    chars: &[char],
    opts: &Options,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) {
    // Same as parse_ws_and_comments but without emitting whitespace into output
    loop {
        let start = *i;
        // fast-skip: if a run of spaces/tabs before a newline, jump directly
        if let Some(&c0) = chars.get(*i) {
            if c0 == ' ' || c0 == '\t' {
                if let Some(start_b) = offs.get(*i).copied() {
                    if let Some(rel) = memchr2(b'\n', b'\r', bytes.get(start_b..).unwrap_or(&[])) {
                        let line = &bytes[start_b..start_b + rel];
                        if line.iter().all(|&b| b == b' ' || b == b'\t') {
                            let new_b = start_b + rel + 1;
                            *i = byte_to_char_index(ascii, offs, new_b, chars.len());
                        } else {
                            // fallback to char loop
                            while let Some(&c) = chars.get(*i) {
                                if is_whitespace(c) {
                                    *i += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                    } else {
                        while let Some(&c) = chars.get(*i) {
                            if is_whitespace(c) {
                                *i += 1;
                            } else {
                                break;
                            }
                        }
                    }
                } else {
                    while let Some(&c) = chars.get(*i) {
                        if is_whitespace(c) {
                            *i += 1;
                        } else {
                            break;
                        }
                    }
                }
            } else {
                while let Some(&c) = chars.get(*i) {
                    if is_whitespace(c) {
                        *i += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        if let Some(&'/') = chars.get(*i) {
            if matches!(chars.get(*i + 1), Some('/')) {
                let start_b = offs.get(*i).copied().unwrap_or(0) + 2;
                let mut new_b = start_b;
                if let Some(rel) = memchr2(b'\n', b'\r', bytes.get(start_b..).unwrap_or(&[])) {
                    new_b = start_b + rel + 1;
                }
                *i = byte_to_char_index(ascii, offs, new_b, chars.len());
                continue;
            }
            if matches!(chars.get(*i + 1), Some('*')) {
                let start_b = offs.get(*i).copied().unwrap_or(0) + 2;
                let mut scan = start_b;
                while let Some(pos) = memchr(b'*', bytes.get(scan..).unwrap_or(&[])) {
                    let idx = scan + pos;
                    if idx + 1 < bytes.len() && bytes[idx + 1] == b'/' {
                        scan = idx + 2;
                        break;
                    }
                    scan = idx + 1;
                }
                *i = byte_to_char_index(ascii, offs, scan, chars.len());
                continue;
            }
        }
        if opts.tolerate_hash_comments
            && let Some(&'#') = chars.get(*i)
        {
            let start_b = offs.get(*i).copied().unwrap_or(0) + 1;
            let mut new_b = start_b;
            if let Some(rel) = memchr2(b'\n', b'\r', bytes.get(start_b..).unwrap_or(&[])) {
                new_b = start_b + rel + 1;
            }
            *i = byte_to_char_index(ascii, offs, new_b, chars.len());
            continue;
        }
        if *i == start {
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_object(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    logger: &mut Logger,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) -> Result<bool, RepairError> {
    if !matches!(chars.get(*i), Some('{')) {
        return Ok(false);
    }
    out.push_char('{');
    *i += 1;
    parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);

    let mut first = true;
    while *i < chars.len() {
        let _ = fast_forward_spaces_to_next_delim(i, chars, bytes, offs, b",}\n\r", ascii);
        if matches!(chars.get(*i), Some('}')) {
            out.push_char('}');
            *i += 1;
            return Ok(true);
        }

        if !first {
            // comma between members; if missing and next token looks like a key, insert
            if matches!(chars.get(*i), Some(',')) {
                out.push_char(',');
                *i += 1;
            } else {
                out.push_char(',');
                logger.log(*i, "inserted missing comma");
            }
            parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
        } else {
            first = false;
        }

        // skip ellipsis like ...
        if skip_ellipsis(i, chars) {
            if matches!(out.last_char(), Some(',')) {
                let _ = out.pop_last();
            }
            continue;
        }

        // Optionally skip word-style comment markers like COMMENT/SHOULD_NOT_EXIST
        while skip_word_comment(i, chars, opts, bytes, offs, ascii) {
            parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
        }

        // key: parse into temp to reuse for path logging
        let mut key_buf = String::new();
        let key_ok = parse_string_any(i, chars, &mut key_buf, /*is_key*/ true, opts, logger)?
            || parse_unquoted(
                i,
                chars,
                &mut key_buf,
                /*is_key*/ true,
                opts,
                logger,
                bytes,
                offs,
                ascii,
            );
        out.push_str(&key_buf);
        if logger.track_path {
            logger.push_key_literal(&key_buf);
        }
        if !key_ok {
            // trim trailing whitespace
            while matches!(out.last_char(), Some(' ' | '\t' | '\n' | '\r')) {
                let _ = out.pop_last();
            }
            if matches!(out.last_char(), Some(',')) {
                let _ = out.pop_last();
            }
            if opts.aggressive_truncation_fix {
                out.push_char('}');
                logger.log_with_context(
                    *i,
                    "aggressively closed object at truncated key",
                    build_context(chars, *i, opts.log_context_window),
                );
                return Ok(true);
            }
            break;
        }

        parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
        // fast: if only spaces/tabs between here and ':', jump directly
        let _ = fast_forward_spaces_to_char(i, chars, bytes, offs, b':', ascii);
        // colon
        if matches!(chars.get(*i), Some(':')) {
            out.push_char(':');
            *i += 1;
        } else {
            out.push_char(':');
            logger.log_with_context(
                *i,
                "inserted missing colon",
                build_context(chars, *i, opts.log_context_window),
            );
        }

        // value
        let parsed = parse_value(i, chars, out, opts, logger, bytes, offs, ascii)?;
        if !parsed {
            if opts.aggressive_truncation_fix {
                // close object early
                if matches!(out.last_char(), Some(',')) {
                    let _ = out.pop_last();
                }
                out.push_char('}');
                logger.log_with_context(
                    *i,
                    "aggressively closed object at truncated value",
                    build_context(chars, *i, opts.log_context_window),
                );
                return Ok(true);
            } else {
                out.push_str("null");
                logger.log_with_context(
                    *i,
                    "inserted null value",
                    build_context(chars, *i, opts.log_context_window),
                );
            }
        }
        out.flush_hint();
        parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
        if logger.track_path {
            logger.pop();
        }
    }
    // missing closing brace
    out.push_char('}');
    logger.log_with_context(
        *i,
        "inserted missing '}'",
        build_context(chars, *i, opts.log_context_window),
    );
    out.flush_hint();
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn parse_array(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    logger: &mut Logger,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) -> Result<bool, RepairError> {
    if !matches!(chars.get(*i), Some('[')) {
        return Ok(false);
    }
    out.push_char('[');
    *i += 1;
    parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);

    let mut first = true;
    let mut elem_index: usize = 0;
    while *i < chars.len() {
        let _ = fast_forward_spaces_to_next_delim(i, chars, bytes, offs, b",]\n\r", ascii);
        if matches!(chars.get(*i), Some(']')) {
            out.push_char(']');
            *i += 1;
            return Ok(true);
        }

        if !first {
            if matches!(chars.get(*i), Some(',')) {
                out.push_char(',');
                *i += 1;
            } else {
                out.push_char(',');
                logger.log_with_context(
                    *i,
                    "inserted missing comma",
                    build_context(chars, *i, opts.log_context_window),
                );
            }
        } else {
            first = false;
        }

        parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
        if skip_ellipsis(i, chars) {
            if matches!(out.last_char(), Some(',')) {
                let _ = out.pop_last();
            }
            logger.log_with_context(
                *i,
                "skipped ellipsis",
                build_context(chars, *i, opts.log_context_window),
            );
            continue;
        }
        logger.push_index(elem_index);
        let parsed = parse_value(i, chars, out, opts, logger, bytes, offs, ascii)?;
        logger.pop();
        elem_index += 1;
        if !parsed {
            // trailing comma -> remove last comma and stop
            if matches!(out.last_char(), Some(',')) {
                let _ = out.pop_last();
            }
            if opts.aggressive_truncation_fix {
                out.push_char(']');
                logger.log_with_context(
                    *i,
                    "aggressively closed array at truncated element",
                    build_context(chars, *i, opts.log_context_window),
                );
                return Ok(true);
            } else {
                break;
            }
        }
        out.flush_hint();
        parse_ws_and_comments(i, chars, out, opts, logger, bytes, offs, ascii);
    }
    // missing closing bracket
    out.push_char(']');
    logger.log_with_context(
        *i,
        "inserted missing ']' ",
        build_context(chars, *i, opts.log_context_window),
    );
    out.flush_hint();
    Ok(true)
}

fn parse_string_any(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    is_key: bool,
    opts: &Options,
    logger: &mut Logger,
) -> Result<bool, RepairError> {
    let Some(&q) = chars.get(*i) else {
        return Ok(false);
    };
    if !is_quote(q) {
        return Ok(false);
    }
    *i += 1; // skip opening quote
    out.push_char('"');
    if q != '"' {
        logger.log_with_context(
            *i,
            "normalized start quote to double",
            build_context(chars, *i, opts.log_context_window),
        );
    }

    // allowed delimiters to early-stop if missing end quote
    let stop_delims: &[char] = if is_key {
        &[':', '}', ',']
    } else {
        &[',', '}', ']', '\n', '\r']
    };

    while *i < chars.len() {
        let c = chars[*i];
        if c == q {
            // found end quote in source
            *i += 1;
            out.push_char('"');
            return Ok(true);
        }
        if c == '"' {
            // raw double quote inside -> escape
            out.push_str("\\\"");
            *i += 1;
            continue;
        }
        if c == '\\' {
            // escape sequence: copy conservatively
            // best-effort: copy next char if exists
            out.push_char('\\');
            *i += 1;
            if let Some(&n) = chars.get(*i) {
                out.push_char(n);
                *i += 1;
            }
            continue;
        }
        if stop_delims.contains(&c) {
            // missing end quote -> close now (do not consume delimiter)
            out.push_char('"');
            logger.log_with_context(
                *i,
                "inserted missing end quote",
                build_context(chars, *i, opts.log_context_window),
            );
            return Ok(true);
        }
        // control characters -> escape basic ones
        push_json_char(out, c, opts.ensure_ascii);
        *i += 1;
    }
    // EOF: close quote
    out.push_char('"');
    logger.log_with_context(
        *i,
        "inserted missing end quote",
        build_context(chars, *i, opts.log_context_window),
    );
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn parse_unquoted(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    is_key: bool,
    opts: &Options,
    logger: &mut Logger,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) -> bool {
    let Some(&c0) = chars.get(*i) else {
        return false;
    };
    // symbol start: letter, underscore, $, digit (for keys we allow digits too -> will be quoted)
    if !(c0.is_alphabetic() || c0 == '_' || c0 == '$' || c0.is_ascii_digit()) {
        return false;
    }

    let start = *i;
    // Fast path: jump to next delimiter using bytes, then map back to char index.
    if let Some(start_b) = offs.get(*i).copied() {
        if let Some(rel) = find_next_unquoted_break(bytes.get(start_b..).unwrap_or(&[]), is_key) {
            let new_b = start_b + rel;
            *i = byte_to_char_index(ascii, offs, new_b, chars.len());
        } else {
            // no delimiter found: jump to end
            *i = chars.len();
        }
    } else {
        // Fallback: char-by-char (should be rare)
        while *i < chars.len() {
            let c = chars[*i];
            if is_unquoted_string_delimiter(c) || (is_key && c == ':') || is_quote(c) {
                break;
            }
            *i += 1;
        }
    }

    if *i == start {
        return false;
    }
    let symbol: String = chars[start..*i].iter().collect();

    // Lookahead for function call like callback(...), NumberLong("2")
    let mut k = *i;
    // skip whitespace/comments silently
    parse_ws_and_comments_silent(&mut k, chars, opts, bytes, offs, ascii);
    if matches!(chars.get(k), Some('(')) {
        // Skip '('
        k += 1;
        *i = k;
        // Parse inner value; append only the inner JSON
        let mut dummy = Logger {
            enable: false,
            entries: Vec::new(),
            track_path: false,
            path: Vec::new(),
        };
        let _ = parse_value(i, chars, out, opts, &mut dummy, bytes, offs, ascii);
        // Skip trailing whitespace/comments and optional ')', then optional ';'
        parse_ws_and_comments_silent(i, chars, opts, bytes, offs, ascii);
        if matches!(chars.get(*i), Some(')')) {
            *i += 1;
        }
        if matches!(chars.get(*i), Some(';')) {
            *i += 1;
        }
        return true;
    }
    if !is_key {
        if opts.repair_undefined && symbol == "undefined" {
            out.push_str("null");
            logger.log_with_context(
                *i,
                "replaced undefined with null",
                build_context(chars, *i, opts.log_context_window),
            );
            return true;
        }
        if symbol == "true" || symbol == "false" || symbol == "null" {
            out.push_str(&symbol);
            return true;
        }
        if opts.allow_python_keywords {
            if symbol == "True" {
                out.push_str("true");
                logger.log_with_context(
                    *i,
                    "normalized python keyword",
                    build_context(chars, *i, opts.log_context_window),
                );
                return true;
            }
            if symbol == "False" {
                out.push_str("false");
                logger.log_with_context(
                    *i,
                    "normalized python keyword",
                    build_context(chars, *i, opts.log_context_window),
                );
                return true;
            }
            if symbol == "None" {
                out.push_str("null");
                logger.log_with_context(
                    *i,
                    "normalized python keyword",
                    build_context(chars, *i, opts.log_context_window),
                );
                return true;
            }
        }
    }
    // write as JSON string
    out.push_char('"');
    for ch in symbol.chars() {
        push_json_char(out, ch, opts.ensure_ascii);
    }
    out.push_char('"');
    if is_key {
        logger.log_with_context(
            *i,
            "quoted unquoted key",
            build_context(chars, *i, opts.log_context_window),
        );
    } else {
        logger.log_with_context(
            *i,
            "quoted unquoted value",
            build_context(chars, *i, opts.log_context_window),
        );
    }
    // if we had a missing start quote and next is end quote, skip it
    if matches!(chars.get(*i), Some('"')) {
        *i += 1;
    }
    true
}

fn parse_keywords(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    logger: &mut Logger,
) -> bool {
    // JSON keywords always allowed
    if starts_with_at(chars, *i, "true") {
        out.push_str("true");
        *i += 4;
        return true;
    }
    if starts_with_at(chars, *i, "false") {
        out.push_str("false");
        *i += 5;
        return true;
    }
    if starts_with_at(chars, *i, "null") {
        out.push_str("null");
        *i += 4;
        return true;
    }
    // Python style keywords only when allowed
    if opts.allow_python_keywords {
        if starts_with_at(chars, *i, "True") {
            out.push_str("true");
            *i += 4;
            logger.log_with_context(
                *i,
                "normalized python keyword",
                build_context(chars, *i, opts.log_context_window),
            );
            return true;
        }
        if starts_with_at(chars, *i, "False") {
            out.push_str("false");
            *i += 5;
            logger.log_with_context(
                *i,
                "normalized python keyword",
                build_context(chars, *i, opts.log_context_window),
            );
            return true;
        }
        if starts_with_at(chars, *i, "None") {
            out.push_str("null");
            *i += 4;
            logger.log_with_context(
                *i,
                "normalized python keyword",
                build_context(chars, *i, opts.log_context_window),
            );
            return true;
        }
    }
    false
}

pub(crate) fn repair_to_string_with_log(
    input: &str,
    opts: &Options,
) -> Result<(String, Vec<RepairLogEntry>), RepairError> {
    // Execute with a live logger and return collected entries
    let mut out = String::with_capacity(input.len() + 8);
    let chars: Vec<char> = input.chars().collect();
    let bytes = input.as_bytes();
    let ascii = input.is_ascii();
    let mut char_offsets: Vec<usize> = Vec::with_capacity(chars.len());
    for (bi, _c) in input.char_indices() {
        char_offsets.push(bi);
    }
    let mut i: usize = 0;
    let mut logger = Logger {
        enable: true,
        entries: Vec::new(),
        track_path: opts.log_json_path,
        path: Vec::new(),
    };

    // Skip BOM then possible fenced start
    skip_bom(&chars, &mut i);
    let mut content_start = i;
    if opts.fenced_code_blocks && skip_markdown_fence_start(&mut i, &chars) {
        content_start = i;
        logger.log(i, "skipped fenced code block start");
    }

    parse_value(
        &mut i,
        &chars,
        &mut out,
        opts,
        &mut logger,
        bytes,
        &char_offsets,
        ascii,
    )?;
    parse_ws_and_comments(
        &mut i,
        &chars,
        &mut out,
        opts,
        &mut logger,
        bytes,
        &char_offsets,
        ascii,
    );

    if matches!(chars.get(i), Some(',')) {
        i += 1;
        parse_ws_and_comments(
            &mut i,
            &chars,
            &mut out,
            opts,
            &mut logger,
            bytes,
            &char_offsets,
            ascii,
        );
    }
    let is_next_value = chars
        .get(i)
        .copied()
        .map(is_start_of_value)
        .unwrap_or(false);
    if is_next_value {
        let mut nd_out = String::with_capacity(out.len() + 8);
        nd_out.push('[');
        let mut j = content_start;
        let mut first = true;
        while j < chars.len() {
            parse_ws_and_comments_silent(&mut j, &chars, opts, bytes, &char_offsets, ascii);
            if j >= chars.len() {
                break;
            }
            if matches!(chars.get(j), Some('}' | ']')) {
                j += 1;
                continue;
            }
            if !chars
                .get(j)
                .copied()
                .map(is_start_of_value)
                .unwrap_or(false)
            {
                break;
            }
            if !first {
                nd_out.push(',');
            }
            first = false;
            let mut elem = String::new();
            parse_value(
                &mut j,
                &chars,
                &mut elem,
                opts,
                &mut logger,
                bytes,
                &char_offsets,
                ascii,
            )?;
            nd_out.push_str(&elem);
            parse_ws_and_comments_silent(&mut j, &chars, opts, bytes, &char_offsets, ascii);
            if matches!(chars.get(j), Some(',')) {
                j += 1;
            }
        }
        nd_out.push(']');
        i = j;
        if opts.fenced_code_blocks && skip_markdown_fence_end(&mut i, &chars) {
            logger.log(i, "skipped fenced code block end");
        }
        logger.log(i, "wrapped NDJSON root array");
        return Ok((nd_out, logger.entries));
    }

    while let Some(c) = chars.get(i) {
        if matches!(c, '}' | ']') {
            i += 1;
            parse_ws_and_comments(
                &mut i,
                &chars,
                &mut out,
                opts,
                &mut logger,
                bytes,
                &char_offsets,
                ascii,
            );
        } else {
            break;
        }
    }
    while let Some(&c) = chars.get(i) {
        if is_whitespace(c) {
            out.push(c);
            i += 1;
        } else {
            break;
        }
    }
    if opts.fenced_code_blocks && skip_markdown_fence_end(&mut i, &chars) {
        logger.log(i, "skipped fenced code block end");
    }
    Ok((out, logger.entries))
}

fn parse_number(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
) -> Result<bool, RepairError> {
    let Some(&c0) = chars.get(*i) else {
        return Ok(false);
    };
    // Accept '-' or digit; optionally accept leading '.' when enabled
    if !(c0.is_ascii_digit() || c0 == '-' || (c0 == '.' && opts.number_tolerance_leading_dot)) {
        return Ok(false);
    }
    let mut s = String::new();
    // sign
    if matches!(chars.get(*i), Some('-')) {
        s.push('-');
        *i += 1;
    }
    // integer part or leading dot tolerance
    let mut has_digit = false;
    if matches!(chars.get(*i), Some('.')) && opts.number_tolerance_leading_dot {
        // treat .25 as 0.25
        s.push_str("0.");
        *i += 1;
        while let Some(&c) = chars.get(*i) {
            if c.is_ascii_digit() {
                s.push(c);
                *i += 1;
                has_digit = true;
            } else {
                break;
            }
        }
    } else {
        // integer digits
        while let Some(&c) = chars.get(*i) {
            if c.is_ascii_digit() {
                s.push(c);
                *i += 1;
                has_digit = true;
            } else {
                break;
            }
        }
        // fraction
        if matches!(chars.get(*i), Some('.')) {
            s.push('.');
            *i += 1;
            let mut frac_cnt = 0usize;
            while let Some(&c) = chars.get(*i) {
                if c.is_ascii_digit() {
                    s.push(c);
                    *i += 1;
                    frac_cnt += 1;
                } else {
                    break;
                }
            }
            if frac_cnt == 0 && opts.number_tolerance_trailing_dot {
                s.push('0');
            }
        }
    }
    // exponent
    if let Some(&e) = chars.get(*i)
        && (e == 'e' || e == 'E')
    {
        let pos_before_e = s.len();
        // consume 'e' / 'E'
        s.push(e);
        *i += 1;
        if let Some(&sg) = chars.get(*i)
            && (sg == '+' || sg == '-')
        {
            s.push(sg);
            *i += 1;
        }
        let mut exp_cnt = 0usize;
        while let Some(&c) = chars.get(*i) {
            if c.is_ascii_digit() {
                s.push(c);
                *i += 1;
                exp_cnt += 1;
            } else {
                break;
            }
        }
        if exp_cnt == 0 && opts.number_tolerance_incomplete_exponent {
            // drop exponent entirely from output but consume the 'e' and optional sign from input
            s.truncate(pos_before_e);
        }
    }
    if !has_digit {
        return Ok(false);
    }
    // leading zero policy
    let s_abs = if let Some(rest) = s.strip_prefix('-') {
        rest
    } else {
        s.as_str()
    };
    let leading_zero =
        s_abs.len() >= 2 && s_abs.as_bytes()[0] == b'0' && s_abs.as_bytes()[1].is_ascii_digit();
    if leading_zero && matches!(opts.leading_zero_policy, LeadingZeroPolicy::QuoteAsString) {
        out.push_char('"');
        out.push_str(&s);
        out.push_char('"');
    } else {
        out.push_str(&s);
    }
    Ok(true)
}

fn starts_with_at(chars: &[char], pos: usize, s: &str) -> bool {
    let mut j = pos;
    for ch in s.chars() {
        if !matches!(chars.get(j), Some(&c) if c==ch) {
            return false;
        }
        j += 1;
    }
    true
}

fn parse_regex(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
) -> Result<bool, RepairError> {
    if !matches!(chars.get(*i), Some('/')) {
        return Ok(false);
    }
    let start = *i;
    *i += 1;
    while *i < chars.len() {
        let c = chars[*i];
        if c == '/' && chars.get(*i - 1) != Some(&'\\') {
            *i += 1;
            break;
        }
        *i += 1;
    }
    // wrap as JSON string
    out.push_char('"');
    for ch in chars[start..*i].iter().copied() {
        push_json_char(out, ch, opts.ensure_ascii);
    }
    out.push_char('"');
    Ok(true)
}

fn parse_concatenated_string(
    i: &mut usize,
    chars: &[char],
    out: &mut dyn Out,
    opts: &Options,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) -> Result<(), RepairError> {
    // After a string, handle patterns like "hello" + "world" by concatenating content
    loop {
        // consume whitespace/comments
        parse_ws_and_comments_silent(i, chars, opts, bytes, offs, ascii);
        if !matches!(chars.get(*i), Some('+')) {
            break;
        }
        *i += 1; // skip '+'
        parse_ws_and_comments_silent(i, chars, opts, bytes, offs, ascii);
        // The output must end with '"' now; remove it to append next content
        if !matches!(out.last_char(), Some('"')) {
            break;
        }
        let _ = out.pop_last();
        // Parse next string literal content and append
        if let Some(content) = parse_string_literal_content(i, chars, opts)? {
            out.push_str(&content);
            // close quote again
            out.push_char('"');
        } else {
            // no valid string after '+', restore and stop
            out.push_char('"');
            break;
        }
    }
    Ok(())
}

fn parse_string_literal_content(
    i: &mut usize,
    chars: &[char],
    opts: &Options,
) -> Result<Option<String>, RepairError> {
    let Some(&q) = chars.get(*i) else {
        return Ok(None);
    };
    if q != '"' && q != '\u{27}' {
        return Ok(None);
    }
    *i += 1;
    let mut buf = String::new();
    while *i < chars.len() {
        let c = chars[*i];
        if c == q {
            *i += 1;
            return Ok(Some(buf));
        }
        if c == '\\' {
            // copy escape as-is (minimal), try to include next char
            if let Some(&n) = chars.get(*i + 1) {
                buf.push('\u{5C}');
                buf.push(n);
                *i += 2;
            } else {
                *i += 1;
            }
            continue;
        }
        if c == '"' {
            buf.push_str("\\\"");
        } else {
            push_json_char(&mut buf, c, opts.ensure_ascii);
        }
        *i += 1;
    }
    // EOF: treat as closed
    Ok(Some(buf))
}

#[inline]
fn push_json_char(out: &mut dyn Out, c: char, ensure_ascii: bool) {
    match c {
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\u{0008}' => out.push_str("\\b"),
        '\u{000C}' => out.push_str("\\f"),
        _ => {
            if ensure_ascii && (c as u32) > 0x7F {
                encode_unicode_escape(out, c);
            } else {
                out.push_char(c);
            }
        }
    }
}

#[inline]
fn encode_unicode_escape(out: &mut dyn Out, c: char) {
    let code = c as u32;
    if code <= 0xFFFF {
        out.push_str(&format!("\\u{:04X}", code));
    } else {
        // Encode as surrogate pair
        let codep = code - 0x1_0000;
        let high = 0xD800 + ((codep >> 10) & 0x3FF);
        let low = 0xDC00 + (codep & 0x3FF);
        out.push_str(&format!("\\u{:04X}\\u{:04X}", high, low));
    }
}

#[inline]
fn skip_ellipsis(i: &mut usize, chars: &[char]) -> bool {
    if *i + 2 >= chars.len() {
        return false;
    }
    if chars[*i] == '.' && chars[*i + 1] == '.' && chars[*i + 2] == '.' {
        *i += 3;
        true
    } else {
        false
    }
}

fn skip_markdown_fence_start(i: &mut usize, chars: &[char]) -> bool {
    // Skip an opening fenced block marker like ```json
    if *i + 2 >= chars.len() {
        return false;
    }
    if !(chars[*i] == '`' && chars[*i + 1] == '`' && chars[*i + 2] == '`') {
        return false;
    }
    *i += 3;
    // skip optional language token
    while *i < chars.len() {
        let c = chars[*i];
        if c.is_ascii_alphabetic() {
            *i += 1;
        } else {
            break;
        }
    }
    // skip a single optional newline after fence
    if matches!(chars.get(*i), Some('\n' | '\r')) {
        *i += 1;
    }
    true
}

fn skip_markdown_fence_end(i: &mut usize, chars: &[char]) -> bool {
    // Skip a closing fenced block marker at current index
    if *i + 2 >= chars.len() {
        return false;
    }
    if !(chars[*i] == '`' && chars[*i + 1] == '`' && chars[*i + 2] == '`') {
        return false;
    }
    *i += 3;
    // skip trailing whitespace/newline
    while let Some(&c) = chars.get(*i) {
        if is_whitespace(c) {
            *i += 1;
        } else {
            break;
        }
    }
    true
}
fn skip_word_comment(
    i: &mut usize,
    chars: &[char],
    opts: &Options,
    bytes: &[u8],
    offs: &[usize],
    ascii: bool,
) -> bool {
    if opts.word_comment_markers.is_empty() {
        return false;
    }
    let Some(&c0) = chars.get(*i) else {
        return false;
    };
    if !(c0.is_alphabetic() || c0 == '_' || c0 == '$' || c0.is_ascii_digit()) {
        return false;
    }
    let start = *i;
    if let Some(start_b) = offs.get(*i).copied() {
        if let Some(rel) = find_next_unquoted_break(bytes.get(start_b..).unwrap_or(&[]), false) {
            let new_b = start_b + rel;
            *i = byte_to_char_index(ascii, offs, new_b, chars.len());
        } else {
            *i = chars.len();
        }
    } else {
        while *i < chars.len() {
            let c = chars[*i];
            if is_unquoted_string_delimiter(c)
                || is_quote(c)
                || c == ':'
                || c == ','
                || c == '}'
                || c == ']'
            {
                break;
            }
            *i += 1;
        }
    }
    if *i == start {
        return false;
    }
    let symbol: String = chars[start..*i].iter().collect();
    let sym_trim = symbol.trim();
    if opts.word_comment_markers.iter().any(|m| m == sym_trim) {
        return true;
    }
    // Not a marker: rewind to start
    *i = start;
    false
}
