#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use crate::emit::{Emitter, JRResult};
use super::lex::skip_ws_and_comments;

/// Parse string literal with optional concatenations or embedded ident-quote form.
/// 🚀 Optimized: use fast byte-level scanning to check for concatenation.
/// Fast path (no concat): parse once and emit.
/// Slow path (has concat): parse and concatenate.
pub fn parse_string_literal_concat_fast<E: Emitter>(
    input: &mut &str,
    opts: &crate::options::Options,
    out: &mut E,
) -> JRResult<()> {
    let s = *input;

    // 🚀 Quick check: is this even a string?
    let quote = match s.as_bytes().first() {
        Some(&b'"') => b'"',
        Some(&b'\'') => b'\'',
        _ => return Ok(()),
    };

    // 🚀 Fast scan to find the end of the string (byte-level)
    let bytes = s.as_bytes();
    let mut i = 1usize; // skip opening quote
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if b == b'\\' {
            escape = true;
            i += 1;
            continue;
        }
        if b == quote {
            i += 1; // include closing quote
            break;
        }
        // Handle multi-byte UTF-8 characters
        if b >= 0x80 {
            // Multi-byte character, skip it
            let ch = s[i..].chars().next().unwrap();
            i += ch.len_utf8();
        } else {
            i += 1;
        }
    }

    // 🚀 Peek ahead after the string to check for concatenation
    let after_string = &s[i..];
    let mut look = after_string;
    skip_ws_and_comments(&mut look, opts);

    // Quick byte-level check for '+' concatenation
    let has_concat = look.as_bytes().first() == Some(&b'+');

    // Check for embedded pattern: <ident><quote> (only if no '+')
    let has_embed = if !has_concat {
        // Fast ASCII identifier check
        let look_bytes = look.as_bytes();
        let mut id_end = 0usize;
        while id_end < look_bytes.len() {
            let b = look_bytes[id_end];
            if id_end == 0 {
                if !(b.is_ascii_alphabetic() || b == b'_' || b == b'$') {
                    break;
                }
            } else if !(b.is_ascii_alphanumeric() || b == b'_' || b == b'$') {
                break;
            }
            id_end += 1;
        }
        if id_end > 0 && id_end < look_bytes.len() {
            let next_b = look_bytes[id_end];
            next_b == b'"' || next_b == b'\''
        } else {
            false
        }
    } else {
        false
    };

    // 🚀 Fast path - no concatenation, parse once and emit
    if !has_concat && !has_embed {
        let lit = parse_one_string_literal(input)?;
        return emit_json_string_from_lit(out, &lit, opts.ensure_ascii);
    }

    // 🔴 Slow path - has concatenation, use temporary buffer
    let lit = parse_one_string_literal(input)?;
    let mut acc = String::new();
    acc.push_str(&lit);
    *input = after_string;
    finish_string_concat(input, opts, out, acc)
}

/// Helper to finish string concatenation after the first string is already parsed
fn finish_string_concat<E: Emitter>(
    input: &mut &str,
    opts: &crate::options::Options,
    out: &mut E,
    mut acc: String,
) -> JRResult<()> {
    loop {
        skip_ws_and_comments(input, opts);
        if let Some(r) = input.strip_prefix('+') {
            *input = r;
            skip_ws_and_comments(input, opts);
            let lit2 = parse_one_string_literal(input)?;
            acc.push_str(&lit2);
            continue;
        }

        let sref = *input;
        let mut id_end = 0usize;
        for (i, ch) in sref.char_indices() {
            if i == 0 {
                if !(ch.is_ascii_alphabetic() || ch == '_' || ch == '$') { break; }
                id_end = i + ch.len_utf8();
            } else {
                if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') { break; }
                id_end = i + ch.len_utf8();
            }
        }
        if id_end > 0 {
            let ident = &sref[..id_end];
            let rest = &sref[id_end..];
            if let Some(q) = rest.chars().next() {
                if q == '"' || q == '\'' {
                    *input = &rest[q.len_utf8()..];
                    acc.push(q);
                    acc.push_str(ident);
                    acc.push(q);
                    let s2 = *input;
                    let mut idx = 0usize;
                    while idx < s2.len() {
                        let ch = s2[idx..].chars().next().unwrap();
                        let l = ch.len_utf8();
                        if ch == q { *input = &s2[idx + l..]; break; }
                        acc.push(ch);
                        idx += l;
                    }
                    continue;
                }
            }
        }
        break;
    }
    emit_json_string_from_lit(out, &acc, opts.ensure_ascii)
}

pub fn parse_one_string_literal(input: &mut &str) -> JRResult<String> {
    let s = *input;
    let mut it = s.char_indices();
    let (start_i, quote) = match it.next() {
        Some((i, '"')) => (i, '"'),
        Some((i, '\'')) => (i, '\''),
        _ => return Ok(String::new()),
    };
    let mut i = start_i + 1;
    let mut out = String::new();
    let _bytes = s.as_bytes();
    let mut escape = false;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        let l = ch.len_utf8();
        i += l;
        if escape {
            escape = false;
            match ch {
                '\\' => out.push('\\'),
                '"' => out.push('"'),
                '\'' => out.push('\''),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000C}'),
                'u' => {
                    // \uXXXX (handle surrogate pairs)
                    if i + 4 <= s.len() {
                        let hex = &s[i..i + 4];
                        if let Ok(v) = u16::from_str_radix(hex, 16) {
                            let is_high = (0xD800..=0xDBFF).contains(&v);
                            let is_low = (0xDC00..=0xDFFF).contains(&v);
                            if !is_high && !is_low {
                                if let Some(c) = char::from_u32(v as u32) {
                                    out.push(c);
                                }
                                i += 4;
                            } else if is_high {
                                // Try to consume a following low surrogate
                                if i + 6 <= s.len() && s[i + 4..].starts_with("\\u") && i + 10 <= s.len() {
                                    let lo_hex = &s[i + 6..i + 10];
                                    if let Ok(lo) = u16::from_str_radix(lo_hex, 16) {
                                        if (0xDC00..=0xDFFF).contains(&lo) {
                                            let hi = v as u32 - 0xD800;
                                            let lo10 = lo as u32 - 0xDC00;
                                            let code = 0x1_0000 + ((hi << 10) | lo10);
                                            if let Some(c) = char::from_u32(code) {
                                                out.push(c);
                                            }
                                            i += 10; // consumed XXXX, "\\u", and next XXXX
                                        }
                                    }
                                }
                                // Fallback: skip high surrogate and emit nothing
                                i += 4;
                            } else {
                                // Isolated low surrogate: skip
                                i += 4;
                            }
                        } else {
                            i += 4; // skip invalid hex
                        }
                    }
                }
                _ => out.push(ch),
            }
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == quote {
            // end
            *input = &s[i..];
            return Ok(out);
        }
        out.push(ch);
    }
    // unclosed string: best-effort close
    *input = &s[s.len()..];
    Ok(out)
}

// Strict variant for object keys: stop at the first matching closing quote.
pub fn parse_one_string_key_strict(input: &mut &str) -> JRResult<String> {
    let s = *input;
    let mut it = s.char_indices();
    let (start_i, quote) = match it.next() {
        Some((i, '"')) => (i, '"'),
        Some((i, '\'')) => (i, '\''),
        _ => return Ok(String::new()),
    };
    let mut i = start_i + 1;
    let mut out = String::new();
    let mut escape = false;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        let l = ch.len_utf8();
        i += l;
        if escape { escape = false; out.push(ch); continue; }
        if ch == '\\' { escape = true; continue; }
        if ch == quote {
            *input = &s[i..];
            return Ok(out);
        }
        out.push(ch);
    }
    *input = &s[s.len()..];
    Ok(out)
}

pub fn emit_json_string_from_lit<E: Emitter>(out: &mut E, s: &str, ensure_ascii: bool) -> JRResult<()> {
    // Fast path: if ASCII-only and contains no characters requiring escaping, write as one slice.
    if s.is_ascii() {
        let bytes = s.as_bytes();
        let mut needs_escape = false;
        for &b in bytes {
            // Must escape: '"', '\\', or control (<= 0x1F)
            if b == b'"' || b == b'\\' || b <= 0x1F {
                needs_escape = true;
                break;
            }
        }
        if !needs_escape {
            out.emit_char('"')?;
            out.emit_str(s)?;
            return out.emit_char('"');
        }
    }

    // General path: stream out safe runs and emit escapes only when needed.
    out.emit_char('"')?;
    let mut start = 0usize; // start of current safe run
    for (i, ch) in s.char_indices() {
        let code = ch as u32;
        let needs_escape =
            ch == '"' || ch == '\\' || code <= 0x1F || (ensure_ascii && code > 0x7F);
        if !needs_escape { continue; }
        // Flush safe run before this char
        if i > start {
            out.emit_str(&s[start..i])?;
        }
        match ch {
            '"' => out.emit_str("\\\"")?,
            '\\' => out.emit_str("\\\\")?,
            '\u{08}' => out.emit_str("\\b")?,
            '\u{0C}' => out.emit_str("\\f")?,
            '\n' => out.emit_str("\\n")?,
            '\r' => out.emit_str("\\r")?,
            '\t' => out.emit_str("\\t")?,
            _ if code <= 0x1F => {
                let esc = format!("\\u{:04X}", code);
                out.emit_str(&esc)?;
            }
            _ => {
                // ensure_ascii && non-ASCII
                debug_assert!(ensure_ascii && code > 0x7F);
                if code <= 0xFFFF {
                    let esc = format!("\\u{:04X}", code);
                    out.emit_str(&esc)?;
                } else {
                    let u = code - 0x1_0000;
                    let hi = 0xD800 + ((u >> 10) & 0x3FF);
                    let lo = 0xDC00 + (u & 0x3FF);
                    let esc = format!("\\u{:04X}\\u{:04X}", hi, lo);
                    out.emit_str(&esc)?;
                }
            }
        }
        start = i + ch.len_utf8();
    }
    if start < s.len() {
        out.emit_str(&s[start..])?;
    }
    out.emit_char('"')
}


