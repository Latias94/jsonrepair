use crate::emit::{Emitter, JRResult};
use super::lex::skip_ws_and_comments;

/// Parse string literal with optional concatenations or embedded ident-quote form.
/// Fast path: when no concatenation/embedding follows, emit current literal directly.
pub fn parse_string_literal_concat_fast<E: Emitter>(
    input: &mut &str,
    opts: &crate::options::Options,
    out: &mut E,
) -> JRResult<()> {
    // Parse first literal (content only, without quotes)
    let lit = parse_one_string_literal(input)?;

    // Probe lookahead for '+' concatenation or embedded ident + quote
    let mut look = *input;
    skip_ws_and_comments(&mut look, opts);
    let no_plus = !look.starts_with('+');
    // Detect embedded pattern: <ident><quote>
    let mut id_end = 0usize;
    for (i, ch) in look.char_indices() {
        if i == 0 {
            if !(ch.is_ascii_alphabetic() || ch == '_' || ch == '$') { break; }
            id_end = i + ch.len_utf8();
        } else {
            if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') { break; }
            id_end = i + ch.len_utf8();
        }
    }
    let has_embed = if id_end > 0 {
        let rest = &look[id_end..];
        matches!(rest.chars().next(), Some('"') | Some('\''))
    } else { false };

    if no_plus && !has_embed {
        return emit_json_string_from_lit(out, &lit, opts.ensure_ascii);
    }

    // Slow path: materialize concatenations
    let mut acc = String::new();
    acc.push_str(&lit);
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
#[allow(dead_code)]
pub fn parse_string_literal_concat<E: Emitter>(input: &mut &str, opts: &crate::options::Options, out: &mut E) -> JRResult<()> {
    // 解析一个字符串字面量，并支持后续的拼接：
    // 1) 使用加号连接："a" + "b"
    // 2) 内嵌引号修复："lorem "ipsum" sic" -> 合并为单个字符串
    let mut acc = String::new();
    let lit = parse_one_string_literal(input)?;
    acc.push_str(&lit);
    loop {
        // 跳过空白和注释，优先处理 + 拼接
        skip_ws_and_comments(input, opts);
        if let Some(r) = input.strip_prefix('+') {
            *input = r;
            skip_ws_and_comments(input, opts);
            let lit2 = parse_one_string_literal(input)?;
            acc.push_str(&lit2);
            continue;
        }
        // 内嵌引号启发式：紧跟未引号的标识符 + 同样的引号 + 下一段字符串
        let sref = *input;
        // 读取标识符
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
                    // 消耗 ident 和一个引号
                    *input = &rest[q.len_utf8()..];
                    // 并入累积字符串："ident"
                    acc.push(q);
                    acc.push_str(ident);
                    acc.push(q);
                    // 继续读取字符串直至下一个匹配引号
                    let s2 = *input; let mut idx = 0usize; while idx < s2.len() { let ch = s2[idx..].chars().next().unwrap(); let l = ch.len_utf8(); if ch == q { *input = &s2[idx + l..]; break; } acc.push(ch); idx += l; }
                    continue;
                }
            }
        }
        break;
    }
    emit_json_string_from_lit(out, &acc, opts.ensure_ascii)
}pub fn parse_one_string_literal(input: &mut &str) -> JRResult<String> {
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




