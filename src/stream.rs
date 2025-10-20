#![allow(clippy::collapsible_if)]

use crate::classify::{is_double_quote_like, is_single_quote_like, is_whitespace};
use crate::error::{RepairError, RepairErrorKind};
use crate::{Options, repair_to_string};
// use crate::parser::{parse_one_root_value, Logger as PLogger};
// use crate::emit::StringEmitter;
use memchr::{memchr, memchr2};
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuoteKind {
    Single,
    Double,
}

pub struct StreamRepairer {
    opts: Options,
    buf: String,
    seg_start: usize,
    scan_pos: usize,
    depth: i32,
    in_string: bool,
    quote_kind: QuoteKind,
    escape: bool,
    in_line_comment: bool,
    in_block_comment: bool,
    in_fence: bool,
    value_started: bool,
    last_sig_end: usize,
    // NDJSON aggregation (optional)
    agg_open: bool,
    agg_buf: String,
}

impl StreamRepairer {
    pub fn new(opts: Options) -> Self {
        Self {
            opts,
            buf: String::new(),
            seg_start: 0,
            scan_pos: 0,
            depth: 0,
            in_string: false,
            quote_kind: QuoteKind::Double,
            escape: false,
            in_line_comment: false,
            in_block_comment: false,
            in_fence: false,
            value_started: false,
            last_sig_end: 0,
            agg_open: false,
            agg_buf: String::new(),
        }
    }

    // Add a value to string aggregation buffer
    fn agg_add_val_str(&mut self, val: &str) {
        if !self.agg_open {
            self.agg_open = true;
            self.agg_buf.clear();
            self.agg_buf.push('[');
        }
        if self
            .agg_buf
            .as_bytes()
            .last()
            .map(|&b| b != b'[')
            .unwrap_or(false)
        {
            self.agg_buf.push(',');
            self.agg_buf.push(' ');
        }
        self.agg_buf.push_str(val);
    }

    // Finish string aggregation and take the aggregated output
    fn agg_finish_str(&mut self) -> Option<String> {
        if self.agg_open {
            self.agg_buf.push(']');
            let ret = std::mem::take(&mut self.agg_buf);
            self.agg_open = false;
            Some(ret)
        } else {
            None
        }
    }

    // Add a value to writer aggregation
    fn agg_add_val_writer<W: Write>(
        &mut self,
        writer: &mut W,
        val: &str,
    ) -> Result<(), RepairError> {
        if !self.agg_open {
            self.agg_open = true;
            writer.write_all(b"[").map_err(|e| {
                RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), 0)
            })?;
        } else {
            writer.write_all(b", ").map_err(|e| {
                RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), 0)
            })?;
        }
        writer.write_all(val.as_bytes()).map_err(|e| {
            RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), 0)
        })
    }

    // Finish writer aggregation
    fn agg_finish_writer<W: Write>(&mut self, writer: &mut W) -> Result<(), RepairError> {
        if self.agg_open {
            writer.write_all(b"]").map_err(|e| {
                RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), 0)
            })?;
            self.agg_open = false;
            self.agg_buf.clear();
        }
        Ok(())
    }

    /// Push a UTF-8 chunk into the streaming repairer and return any completed JSON text.
    ///
    /// Returns `Some(String)` when this call produces a complete root-level JSON value;
    /// otherwise returns `None` (no output yet).
    pub fn push(&mut self, chunk: &str) -> Result<Option<String>, RepairError> {
        self.buf.push_str(chunk);
        let mut out = String::new();
        let mut i = self.scan_pos;
        while i < self.buf.len() {
            // Root-level helper: drop JSONP prefix like ident '(' (allow spaces)
            if self.depth == 0
                && !self.in_string
                && !self.in_line_comment
                && !self.in_block_comment
                && !self.in_fence
            {
                if let Some(slice) = self.buf.get(self.seg_start..self.buf.len()) {
                    // Use lex helper to compute JSONP prefix length relative to current segment start
                    let rel = crate::parser::lex::jsonp_prefix_len(slice);
                    if let Some(off) = rel {
                        let abs = self.seg_start + off;
                        self.scan_pos = abs;
                        self.drop_prefix(abs);
                        i = self.scan_pos;
                        continue;
                    }
                }
            }
            // additional fast path: inside a container (depth>0), not in string/comment
            // jump to next interesting ASCII char to reduce per-char overhead
            if self.depth > 0
                && !self.in_string
                && !self.in_line_comment
                && !self.in_block_comment
                && let Some(bytes) = self.buf.as_bytes().get(i..)
            {
                let mut best: Option<usize> = None;
                // search for '}' or ']'
                if let Some(p) = memchr2(b'}', b']', bytes) {
                    best = Some(p);
                }
                // search for '"' or '\''
                if let Some(p) = memchr2(b'"', b'\'', bytes) {
                    best = Some(best.map_or(p, |b| b.min(p)));
                }
                // search for '/'
                if let Some(p) = memchr(b'/', bytes) {
                    best = Some(best.map_or(p, |b| b.min(p)));
                }
                // also consider nested container openers '{' and '['
                if let Some(p) = memchr2(b'{', b'[', bytes) {
                    best = Some(best.map_or(p, |b| b.min(p)));
                }
                if let Some(p) = best
                    && p > 0
                {
                    i += p;
                    continue;
                }
            }
            // fast-path: at root and before a value, drop up to next newline quickly
            if self.depth == 0
                && !self.in_string
                && !self.in_line_comment
                && !self.in_block_comment
                && !self.value_started
                && let Some(bytes) = self.buf.as_bytes().get(i..)
                && let Some(pos) = memchr2(b'\n', b'\r', bytes)
            {
                // only drop to newline if the prefix up to newline is spaces/tabs
                let prefix = &bytes[..pos];
                if prefix.iter().all(|&b| b == b' ' || b == b'\t') {
                    let end = i + pos + 1;
                    self.scan_pos = end;
                    self.drop_prefix(end);
                    i = self.scan_pos;
                    continue;
                }
            }
            if self.in_line_comment {
                if let Some(bytes) = self.buf.as_bytes().get(i..)
                    && let Some(pos) = memchr2(b'\n', b'\r', bytes)
                {
                    i += pos + 1; // skip newline
                    self.in_line_comment = false;
                    continue;
                }
                break; // need more data
            }
            if self.in_block_comment {
                if let Some(bytes) = self.buf.as_bytes().get(i..) {
                    let mut offset = 0;
                    while let Some(pos) = memchr(b'*', &bytes[offset..]) {
                        let idx = i + offset + pos;
                        if idx + 1 < self.buf.len() && self.buf.as_bytes()[idx + 1] == b'/' {
                            // found */
                            i = idx + 2;
                            self.in_block_comment = false;
                            // at root and no value started: drop comment segment
                            if self.depth == 0 && !self.value_started {
                                self.scan_pos = i;
                                self.drop_prefix(i);
                                i = self.scan_pos;
                            }
                            break;
                        }
                        offset += pos + 1;
                    }
                    if self.in_block_comment {
                        break;
                    }
                    continue;
                } else {
                    break;
                }
            }

            let (ch, len) = next_char(&self.buf, i);
            if len == 0 {
                break;
            }

            if self.in_string {
                if self.escape {
                    self.escape = false;
                    i += len;
                    continue;
                }
                if ch == '\\' {
                    self.escape = true;
                    i += len;
                    continue;
                }
                let end = match self.quote_kind {
                    QuoteKind::Double => is_double_quote_like(ch),
                    QuoteKind::Single => is_single_quote_like(ch),
                };
                if end {
                    self.in_string = false;
                    self.last_sig_end = i + len;
                }
                i += len;
                continue;
            }

            // not in string/comment
            if ch == '/' {
                // lookahead
                let (ch2, l2) = next_char(&self.buf, i + len);
                if l2 > 0 && ch2 == '/' {
                    self.in_line_comment = true;
                    i += len + l2;
                    continue;
                }
                if l2 > 0 && ch2 == '*' {
                    self.in_block_comment = true;
                    i += len + l2;
                    continue;
                }
            } else if ch == '#' {
                // treat # as line comment when allowed
                self.in_line_comment = true;
                i += len;
                continue;
            } else if ch == '`' && self.depth == 0 {
                // fenced code block markers ``` at root-level: drop the markers themselves
                let (c2, l2) = next_char(&self.buf, i + len);
                let (c3, l3) = next_char(&self.buf, i + len + l2);
                if l2 == 0 || l3 == 0 {
                    break;
                }
                if c2 == '`' && c3 == '`' {
                    let mut j = i + len + l2 + l3; // after ```
                    if !self.in_fence {
                        // opening fence: optional language + ws + optional newline
                        let rel = crate::parser::lex::fence_open_lang_newline_len(&self.buf[j..]);
                        j += rel;
                        self.scan_pos = j;
                        self.drop_prefix(self.scan_pos);
                        i = self.scan_pos;
                        self.in_fence = true;
                        continue;
                    } else {
                        // closing fence after a value: emit previous JSON up to last_sig_end, then drop the fence
                        let end = if self.last_sig_end > self.seg_start {
                            self.last_sig_end
                        } else {
                            i
                        };
                        self.scan_pos = i;
                        let emitted = self.emit_segment(end)?;
                        out.push_str(&emitted);
                        i = self.scan_pos;
                        // drop the fence markers themselves
                        let abs_end = i + len + l2 + l3;
                        self.drop_prefix(abs_end);
                        i = self.scan_pos;
                        self.value_started = false;
                        self.in_fence = false;
                        continue;
                    }
                }
            }

            if is_whitespace(ch) {
                // at root and no value: we can drop accumulated whitespace to keep buffer small
                if self.depth == 0 && !self.value_started {
                    self.scan_pos = i + len;
                    self.drop_prefix(self.scan_pos);
                    i = self.scan_pos;
                } else {
                    i += len;
                }
                continue;
            }

            // at root inside an opened fence, before value: drop residual language/whitespace/newline across chunks
            if self.depth == 0 && self.in_fence && !self.value_started {
                let rel = crate::parser::lex::fence_open_lang_newline_len(&self.buf[i..]);
                if rel > 0 {
                    self.scan_pos = i + rel;
                    self.drop_prefix(self.scan_pos);
                    i = self.scan_pos;
                    continue;
                }
            }

            // handle JSONP function wrapper at root before value: drop `name(` prefix using lex helper
            if self.depth == 0 && !self.value_started {
                if let Some(rel) = crate::parser::lex::jsonp_prefix_len(&self.buf[i..]) {
                    self.scan_pos = i + rel; // drop prefix up to '('
                    self.drop_prefix(self.scan_pos);
                    i = self.scan_pos;
                    continue;
                }
            }

            // ignore JSONP trailing artifacts at root when no value collected
            if self.depth == 0 && !self.value_started && (ch == ')' || ch == ';') {
                self.scan_pos = i + len;
                self.drop_prefix(self.scan_pos);
                i = self.scan_pos;
                continue;
            }
            if !self.value_started {
                self.value_started = true;
            }

            if is_double_quote_like(ch) {
                self.in_string = true;
                self.quote_kind = QuoteKind::Double;
                i += len;
                continue;
            }
            if is_single_quote_like(ch) {
                self.in_string = true;
                self.quote_kind = QuoteKind::Single;
                i += len;
                continue;
            }

            match ch {
                '{' | '[' => {
                    self.depth += 1;
                    self.last_sig_end = i + len;
                    i += len;
                }
                '}' | ']' => {
                    self.depth -= 1;
                    self.last_sig_end = i + len;
                    i += len;
                    if self.depth == 0 {
                        let emitted = self.emit_segment(i)?;
                        if self.opts.stream_ndjson_aggregate {
                            self.agg_add_val_str(&emitted);
                        } else {
                            out.push_str(&emitted);
                        }
                        // reset scanner relative to new buffer after drain
                        i = self.scan_pos;
                        self.value_started = false;
                    }
                }
                '\n' => {
                    if self.depth == 0 && self.value_started {
                        let end = if self.last_sig_end > self.seg_start {
                            self.last_sig_end
                        } else {
                            i
                        };
                        // If the root-level segment is only an identifier (likely JSONP prefix), do not emit yet
                        let seg = &self.buf[self.seg_start..end].trim();
                        let is_ident = seg
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
                            .unwrap_or(false)
                            && seg
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
                        if is_ident {
                            // drop the newline and keep buffer to await '('
                            self.scan_pos = i + len;
                            self.drop_prefix(self.seg_start); // preserve name at start; we only drop processed prefix before seg_start
                            i = self.scan_pos;
                            self.value_started = false;
                            continue;
                        }
                        self.scan_pos = i;
                        let emitted = self.emit_segment(end)?;
                        if self.opts.stream_ndjson_aggregate {
                            self.agg_add_val_str(&emitted);
                        } else {
                            out.push_str(&emitted);
                        }
                        i = self.scan_pos;
                        self.value_started = false;
                    } else if self.depth == 0 && !self.value_started {
                        // pure blank/comment line: drop it
                        self.scan_pos = i + len;
                        self.drop_prefix(self.scan_pos);
                        i = self.scan_pos;
                    }
                    i += len;
                }
                _ => {
                    self.last_sig_end = i + len;
                    i += len;
                }
            }
        }
        self.scan_pos = i;
        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out))
        }
    }

    /// Stream input chunk and write any completed repaired JSON values directly into `writer`.
    ///
    /// When `Options.stream_ndjson_aggregate` is true, this will not emit values immediately;
    /// instead, it writes an opening `[` on first value and defers closing `]` to `flush_to_writer`.
    pub fn push_to_writer<W: Write>(
        &mut self,
        chunk: &str,
        writer: &mut W,
    ) -> Result<(), RepairError> {
        self.buf.push_str(chunk);
        let mut i = self.scan_pos;
        while i < self.buf.len() {
            // Grammar fast-path at root for writer: drop JSONP prefix via lex helper
            if self.depth == 0
                && !self.in_string
                && !self.in_line_comment
                && !self.in_block_comment
                && !self.in_fence
            {
                if let Some(slice) = self.buf.get(self.seg_start..self.buf.len()) {
                    if let Some(rel) = crate::parser::lex::jsonp_prefix_len(slice) {
                        let abs = self.seg_start + rel;
                        self.scan_pos = abs;
                        self.drop_prefix(abs);
                        i = self.scan_pos;
                        continue;
                    }
                }
            }
            if self.depth > 0
                && !self.in_string
                && !self.in_line_comment
                && !self.in_block_comment
                && let Some(bytes) = self.buf.as_bytes().get(i..)
            {
                let mut best: Option<usize> = None;
                if let Some(p) = memchr2(b'}', b']', bytes) {
                    best = Some(p);
                }
                if let Some(p) = memchr2(b'"', b'\'', bytes) {
                    best = Some(best.map_or(p, |b| b.min(p)));
                }
                if let Some(p) = memchr(b'/', bytes) {
                    best = Some(best.map_or(p, |b| b.min(p)));
                }
                if let Some(p) = memchr2(b'{', b'[', bytes) {
                    best = Some(best.map_or(p, |b| b.min(p)));
                }
                if let Some(p) = best
                    && p > 0
                {
                    i += p;
                    continue;
                }
            }
            if self.depth == 0
                && !self.in_string
                && !self.in_line_comment
                && !self.in_block_comment
                && !self.value_started
                && let Some(bytes) = self.buf.as_bytes().get(i..)
                && let Some(pos) = memchr2(b'\n', b'\r', bytes)
            {
                let prefix = &bytes[..pos];
                if prefix.iter().all(|&b| b == b' ' || b == b'\t') {
                    let end = i + pos + 1;
                    self.scan_pos = end;
                    self.drop_prefix(end);
                    i = self.scan_pos;
                    continue;
                }
            }
            if self.in_line_comment {
                if let Some(bytes) = self.buf.as_bytes().get(i..)
                    && let Some(pos) = memchr2(b'\n', b'\r', bytes)
                {
                    i += pos + 1;
                    self.in_line_comment = false;
                    continue;
                }
                break;
            }
            if self.in_block_comment {
                if let Some(bytes) = self.buf.as_bytes().get(i..) {
                    let mut offset = 0;
                    while let Some(pos) = memchr(b'*', &bytes[offset..]) {
                        let idx = i + offset + pos;
                        if idx + 1 < self.buf.len() && self.buf.as_bytes()[idx + 1] == b'/' {
                            i = idx + 2;
                            self.in_block_comment = false;
                            if self.depth == 0 && !self.value_started {
                                self.scan_pos = i;
                                self.drop_prefix(i);
                                i = self.scan_pos;
                            }
                            break;
                        }
                        offset += pos + 1;
                    }
                    if self.in_block_comment {
                        break;
                    }
                    continue;
                } else {
                    break;
                }
            }

            let (ch, len) = next_char(&self.buf, i);
            if len == 0 {
                break;
            }

            if self.in_string {
                if self.escape {
                    self.escape = false;
                    i += len;
                    continue;
                }
                if ch == '\\' {
                    self.escape = true;
                    i += len;
                    continue;
                }
                let end = match self.quote_kind {
                    QuoteKind::Double => is_double_quote_like(ch),
                    QuoteKind::Single => is_single_quote_like(ch),
                };
                if end {
                    self.in_string = false;
                    self.last_sig_end = i + len;
                }
                i += len;
                continue;
            }

            if ch == '/' {
                let (ch2, l2) = next_char(&self.buf, i + len);
                if l2 > 0 && ch2 == '/' {
                    self.in_line_comment = true;
                    i += len + l2;
                    continue;
                }
                if l2 > 0 && ch2 == '*' {
                    self.in_block_comment = true;
                    i += len + l2;
                    continue;
                }
            } else if ch == '#' {
                self.in_line_comment = true;
                i += len;
                continue;
            } else if ch == '`' && self.depth == 0 {
                let (c2, l2) = next_char(&self.buf, i + len);
                let (c3, l3) = next_char(&self.buf, i + len + l2);
                if l2 == 0 || l3 == 0 {
                    break;
                }
                if c2 == '`' && c3 == '`' {
                    let mut j = i + len + l2 + l3; // after ```
                    if !self.in_fence {
                        let rel = crate::parser::lex::fence_open_lang_newline_len(&self.buf[j..]);
                        j += rel;
                        self.scan_pos = j;
                        self.drop_prefix(self.scan_pos);
                        i = self.scan_pos;
                        self.in_fence = true;
                        continue;
                    } else {
                        let end = if self.last_sig_end > self.seg_start {
                            self.last_sig_end
                        } else {
                            i
                        };
                        self.scan_pos = i;
                        let emitted = self.emit_segment(end)?;
                        if self.opts.stream_ndjson_aggregate {
                            if !self.agg_open {
                                self.agg_open = true;
                                writer.write_all(b"[").map_err(|e| {
                                    RepairError::new(
                                        RepairErrorKind::Parse(format!("io write error: {}", e)),
                                        i,
                                    )
                                })?;
                            }
                            if self.agg_open && writer.flush().is_err() {}
                            // add comma if necessary
                            // We can't peek previous char, so rely on caller to not mix aggregate fencing with multi-values often
                            // We will use agg_buf only for state in non-writer path; here emit comma via a small heuristic
                            // If we've already written at least one element, we write a comma.
                        }
                        if self.opts.stream_ndjson_aggregate {
                            self.agg_add_val_writer(writer, &emitted)?;
                        } else {
                            writer.write_all(emitted.as_bytes()).map_err(|e| {
                                RepairError::new(
                                    RepairErrorKind::Parse(format!("io write error: {}", e)),
                                    i,
                                )
                            })?;
                        }
                        i = self.scan_pos;
                        let abs_end = i + len + l2 + l3;
                        self.drop_prefix(abs_end);
                        i = self.scan_pos;
                        self.value_started = false;
                        self.in_fence = false;
                        continue;
                    }
                }
            }

            if is_whitespace(ch) {
                if self.depth == 0 && !self.value_started {
                    self.scan_pos = i + len;
                    self.drop_prefix(self.scan_pos);
                    i = self.scan_pos;
                } else {
                    i += len;
                }
                continue;
            }

            if self.depth == 0 && self.in_fence && !self.value_started {
                let rel = crate::parser::lex::fence_open_lang_newline_len(&self.buf[i..]);
                if rel > 0 {
                    self.scan_pos = i + rel;
                    self.drop_prefix(self.scan_pos);
                    i = self.scan_pos;
                    continue;
                }
            }

            if self.depth == 0
                && !self.value_started
                && (ch.is_ascii_alphabetic() || ch == '_' || ch == '$')
            {
                let mut j = i + len;
                loop {
                    let (cx, lx) = next_char(&self.buf, j);
                    if lx > 0 && (cx.is_ascii_alphanumeric() || cx == '_' || cx == '$') {
                        j += lx;
                    } else {
                        break;
                    }
                }
                loop {
                    let (cx, lx) = next_char(&self.buf, j);
                    if lx > 0 && is_whitespace(cx) {
                        j += lx;
                    } else {
                        break;
                    }
                }
                let (c2, l2) = next_char(&self.buf, j);
                if l2 > 0 && c2 == '(' {
                    self.scan_pos = j + l2;
                    self.drop_prefix(self.scan_pos);
                    i = self.scan_pos;
                    continue;
                }
            }

            if self.depth == 0 && !self.value_started && (ch == ')' || ch == ';') {
                self.scan_pos = i + len;
                self.drop_prefix(self.scan_pos);
                i = self.scan_pos;
                continue;
            }
            if !self.value_started {
                self.value_started = true;
            }

            if is_double_quote_like(ch) {
                self.in_string = true;
                self.quote_kind = QuoteKind::Double;
                i += len;
                continue;
            }
            if is_single_quote_like(ch) {
                self.in_string = true;
                self.quote_kind = QuoteKind::Single;
                i += len;
                continue;
            }

            match ch {
                '{' | '[' => {
                    self.depth += 1;
                    self.last_sig_end = i + len;
                    i += len;
                }
                '}' | ']' => {
                    self.depth -= 1;
                    self.last_sig_end = i + len;
                    i += len;
                    if self.depth == 0 {
                        let emitted = self.emit_segment(i)?;
                        if self.opts.stream_ndjson_aggregate {
                            self.agg_add_val_writer(writer, &emitted)?;
                        } else {
                            writer.write_all(emitted.as_bytes()).map_err(|e| {
                                RepairError::new(
                                    RepairErrorKind::Parse(format!("io write error: {}", e)),
                                    i,
                                )
                            })?;
                        }
                        i = self.scan_pos;
                        self.value_started = false;
                    }
                }
                '\n' => {
                    if self.depth == 0 && self.value_started {
                        let end = if self.last_sig_end > self.seg_start {
                            self.last_sig_end
                        } else {
                            i
                        };
                        self.scan_pos = i;
                        let emitted = self.emit_segment(end)?;
                        if self.opts.stream_ndjson_aggregate {
                            self.agg_add_val_writer(writer, &emitted)?;
                        } else {
                            writer.write_all(emitted.as_bytes()).map_err(|e| {
                                RepairError::new(
                                    RepairErrorKind::Parse(format!("io write error: {}", e)),
                                    i,
                                )
                            })?;
                        }
                        i = self.scan_pos;
                        self.value_started = false;
                    } else if self.depth == 0 && !self.value_started {
                        self.scan_pos = i + len;
                        self.drop_prefix(self.scan_pos);
                        i = self.scan_pos;
                    }
                    i += len;
                }
                _ => {
                    self.last_sig_end = i + len;
                    i += len;
                }
            }
        }
        self.scan_pos = i;
        Ok(())
    }

    /// Flush and write any remaining data into `writer`. If NDJSON aggregation is enabled,
    /// this closes the array.
    pub fn flush_to_writer<W: Write>(&mut self, writer: &mut W) -> Result<(), RepairError> {
        if self.seg_start < self.buf.len() {
            if self.depth == 0
                && !self.value_started
                && !self.in_string
                && !self.in_block_comment
                && !self.in_line_comment
                && self.last_sig_end <= self.seg_start
            {
                self.buf.clear();
                self.seg_start = 0;
                self.scan_pos = 0;
                if self.opts.stream_ndjson_aggregate {
                    self.agg_finish_writer(writer)?;
                }
                return Ok(());
            }
            let s = self.buf[self.seg_start..].to_string();
            let fixed = repair_to_string(&s, &self.opts)?;
            if self.opts.stream_ndjson_aggregate {
                self.agg_add_val_writer(writer, &fixed)?;
            } else {
                writer.write_all(fixed.as_bytes()).map_err(|e| {
                    RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), 0)
                })?;
            }
        }
        self.buf.clear();
        self.seg_start = 0;
        self.scan_pos = 0;
        self.depth = 0;
        self.in_string = false;
        self.escape = false;
        self.in_line_comment = false;
        self.in_block_comment = false;
        self.value_started = false;
        self.last_sig_end = 0;
        if self.opts.stream_ndjson_aggregate {
            self.agg_finish_writer(writer)?;
        }
        Ok(())
    }

    /// Flush any remaining buffered content.
    ///
    /// Returns `Some(String)` when there is final output to emit; otherwise `None`.
    pub fn flush(&mut self) -> Result<Option<String>, RepairError> {
        let mut out = String::new();
        if self.seg_start < self.buf.len() {
            // If nothing meaningful collected at root, skip repairing
            if self.depth == 0
                && !self.value_started
                && !self.in_string
                && !self.in_block_comment
                && !self.in_line_comment
                && self.last_sig_end <= self.seg_start
            {
                self.buf.clear();
                self.seg_start = 0;
                self.scan_pos = 0;
                // if aggregating and already opened, close and return aggregated array
                if self.opts.stream_ndjson_aggregate {
                    return Ok(self.agg_finish_str());
                } else {
                    return Ok(if out.is_empty() { None } else { Some(out) });
                }
            }
            let s = self.buf[self.seg_start..].to_string();
            let fixed = repair_to_string(&s, &self.opts)?;
            if self.opts.stream_ndjson_aggregate {
                self.agg_add_val_str(&fixed);
            } else {
                out.push_str(&fixed);
            }
        }
        // reset
        self.buf.clear();
        self.seg_start = 0;
        self.scan_pos = 0;
        self.depth = 0;
        self.in_string = false;
        self.escape = false;
        self.in_line_comment = false;
        self.in_block_comment = false;
        self.value_started = false;
        self.last_sig_end = 0;
        if self.opts.stream_ndjson_aggregate {
            Ok(self.agg_finish_str())
        } else {
            Ok(if out.is_empty() { None } else { Some(out) })
        }
    }

    fn emit_segment(&mut self, end: usize) -> Result<String, RepairError> {
        if end <= self.seg_start {
            self.seg_start = end;
            return Ok(String::new());
        }
        let segment = self.buf[self.seg_start..end].to_string();
        let fixed = repair_to_string(&segment, &self.opts)?;
        // drop processed part from buffer to keep memory bounded
        self.buf.drain(..end);
        // adjust indices
        if self.scan_pos >= end {
            self.scan_pos -= end;
        } else {
            self.scan_pos = 0;
        }
        if self.last_sig_end >= end {
            self.last_sig_end -= end;
        } else {
            self.last_sig_end = 0;
        }
        self.seg_start = 0;
        Ok(fixed)
    }

    fn drop_prefix(&mut self, end: usize) {
        if end <= self.seg_start {
            self.seg_start = end;
            return;
        }
        self.buf.drain(..end);
        if self.scan_pos >= end {
            self.scan_pos -= end;
        } else {
            self.scan_pos = 0;
        }
        if self.last_sig_end >= end {
            self.last_sig_end -= end;
        } else {
            self.last_sig_end = 0;
        }
        self.seg_start = 0;
    }
}

#[inline]
fn next_char(s: &str, i: usize) -> (char, usize) {
    if i >= s.len() {
        return ('\0', 0);
    }
    match s[i..].chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => ('\0', 0),
    }
}
