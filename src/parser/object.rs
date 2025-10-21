#![allow(clippy::needless_lifetimes)]

use super::array::parse_array;
use super::lex::{skip_ellipsis, skip_word_markers, skip_ws_and_comments};
use super::number::parse_number_token;
use super::strings::{
    emit_json_string_from_lit, parse_one_string_key_strict, parse_string_literal_concat_fast,
};
use crate::emit::{Emitter, JRResult};
use crate::options::Options;
use crate::parser::parse_regex_literal;
use crate::parser::parse_symbol_or_unquoted_string;
use memchr::memchr2;

// Helper: if the upcoming content begins with a line comment (// or #),
// cut the comment up to the earliest of newline (\n/\r) or a closing '}'.
// This preserves trailing structural characters so the outer parser can continue,
// matching Python json_repair behavior for inline comments inside objects.
fn skip_line_comment_preserving_rbrace(input: &mut &str, opts: &Options) {
    if input.is_empty() {
        return;
    }
    let s = *input;
    // Trim leading spaces/tabs only; keep newlines intact
    let after_ws = s.trim_start_matches([' ', '\t']);
    // If no comment marker immediately after spaces, return
    let bytes = after_ws.as_bytes();
    if bytes.starts_with(b"//") || (opts.tolerate_hash_comments && bytes.first() == Some(&b'#')) {
        let (_skip, rest) = if bytes.starts_with(b"//") {
            (2usize, &after_ws[2..])
        } else {
            (1usize, &after_ws[1..])
        };
        let rbytes = rest.as_bytes();
        // Find earliest of newline/CR or '}'
        let mut i = 0usize;
        while i < rbytes.len() {
            let b = rbytes[i];
            if b == b'\n' || b == b'\r' || b == b'}' {
                break;
            }
            i += 1;
        }
        if i < rbytes.len() {
            // Start view at the terminator; if newline, drop it
            let mut new_view = &rest[i..];
            if new_view.starts_with('\n') || new_view.starts_with('\r') {
                new_view = &new_view[1..];
            }
            *input = new_view;
        } else {
            // Comment runs to end; nothing more useful
            *input = "";
        }
    }
}

pub fn parse_object<'i, E: Emitter>(
    input: &mut &'i str,
    opts: &Options,
    out: &mut E,
    logger: &mut crate::parser::Logger,
) -> JRResult<()> {
    // assumes current starts with '{'
    if !input.starts_with('{') {
        return Ok(());
    }
    *input = &input[1..];
    out.emit_char('{')?;
    // Enter-object fast path: if only ASCII whitespace before a closing '}', close immediately.
    if let Some('}') = fast_ws_to_only_rbrace(input) {
        out.emit_char('}')?;
        return Ok(());
    }
    skip_ws_and_comments(input, opts);
    let mut first = true;
    loop {
        skip_ws_and_comments(input, opts);
        if input.is_empty() {
            // 截断对象，补全闭合
            out.emit_char('}')?;
            break;
        }
        if input.starts_with(']') {
            out.emit_char('}')?;
            break;
        }
        if input.starts_with('}') {
            *input = &input[1..];
            out.emit_char('}')?;
            break;
        }
        // comma will be emitted later only when a member is actually produced

        // 可选：跳过词注释与省略号
        skip_word_markers(input, &opts.word_comment_markers);
        while skip_ellipsis(input) {
            skip_ws_and_comments(input, opts);
        }
        // Optional comma between members (fast path: ASCII ws -> ',' or '}')
        if let Some(delim) = fast_ws_to_comma_or_rbrace(input) {
            match delim {
                ',' => { /* consumed comma, proceed to next key */ }
                '}' => {
                    out.emit_char('}')?;
                    break;
                }
                _ => unreachable!(),
            }
        } else {
            if input.starts_with(',') {
                *input = &input[1..];
            }
            if input.starts_with('}') {
                *input = &input[1..];
                out.emit_char('}')?;
                break;
            }
        }
        if !first {
            out.emit_char(',')?;
        }
        first = false;

        // key: quoted or unquoted identifier/span until colon/comma/brace
        skip_ws_and_comments(input, opts);
        if input.is_empty() {
            out.emit_char('}')?;
            break;
        }
        let key_str = if input.starts_with('"') || input.starts_with('\'') {
            // For keys, parse literal content for path, then emit as JSON string
            let k = parse_one_string_key_strict(input)?;
            emit_json_string_from_lit(out, &k, opts.ensure_ascii)?;
            k
        } else {
            // Fast path: take until one of ':', '}', ',' or newline via bytes scan
            let key = take_key_until_delim_fast(input)
                .unwrap_or_else(|| take_until_delim(input, &[':', '}', ',']));
            let k = key.trim();
            emit_json_string_from_lit(out, k, opts.ensure_ascii)?;
            k.to_string()
        };
        skip_ws_and_comments(input, opts);
        // colon
        if input.starts_with(':') {
            *input = &input[1..];
            out.emit_char(':')?;
        } else {
            out.emit_char(':')?; // insert missing colon
        }
        skip_ws_and_comments(input, opts);

        // value（可选：再次跳过词注释/省略号）
        if input.is_empty() {
            out.emit_char('}')?;
            break;
        }
        skip_word_markers(input, &opts.word_comment_markers);
        while skip_ellipsis(input) {
            skip_ws_and_comments(input, opts);
        }
        // Track path for value
        logger.push_key(key_str);
        let c = input.chars().next().unwrap();
        match c {
            '{' => super::object::parse_object(input, opts, out, logger)?,
            '[' => parse_array(input, opts, out, logger)?,
            '"' | '\'' => {
                // Heuristic: if a double-quoted value contains a comma before a suspicious close
                // whose following token is not a valid terminator, prefer closing at the comma.
                if c == '"' {
                    let s_val = *input;
                    let mut i = 1usize;
                    let mut escape = false;
                    let mut first_comma: Option<usize> = None;
                    let mut close_pos: Option<usize> = None;
                    while i < s_val.len() {
                        let ch = s_val[i..].chars().next().unwrap();
                        let l = ch.len_utf8();
                        i += l;
                        if escape {
                            escape = false;
                            continue;
                        }
                        if ch == '\\' {
                            escape = true;
                            continue;
                        }
                        if ch == ',' && first_comma.is_none() {
                            first_comma = Some(i - l);
                        }
                        if ch == '"' {
                            close_pos = Some(i - l);
                            break;
                        }
                    }
                    if let Some(cp) = close_pos {
                        // Inspect what follows the quote
                        let mut look = &s_val[cp + 1..];
                        super::lex::skip_ws_and_comments(&mut look, opts);
                        let after_ok = look.is_empty()
                            || look.starts_with(',')
                            || look.starts_with('}')
                            || look.starts_with(']');
                        if !after_ok {
                            if let Some(comma_i) = first_comma {
                                let content = &s_val[1..comma_i];
                                emit_json_string_from_lit(out, content, opts.ensure_ascii)?;
                                // leave input at comma for the outer loop to consume
                                *input = &s_val[comma_i..];
                                logger.pop_key();
                                // Fast path after value handled below; emulate as if value was emitted
                                if let Some(delim) =
                                    super::object::fast_ws_to_comma_or_rbrace(input)
                                {
                                    match delim {
                                        ',' => { /* next member */ }
                                        '}' => {
                                            out.emit_char('}')?;
                                            return Ok(());
                                        }
                                        _ => {}
                                    }
                                }
                                continue;
                            }
                        }
                    }
                }
                parse_string_literal_concat_fast(input, opts, out)?
            }
            '/' => parse_regex_literal(input, opts, out)?,
            c if c == '-' || c == '.' || c.is_ascii_digit() => {
                parse_number_token(input, opts, out)?
            }
            _ => parse_symbol_or_unquoted_string(input, opts, out, logger)?,
        }
        logger.pop_key();

        // Fast path after value: ASCII ws -> next delimiter ',' or '}'
        if let Some(delim) = fast_ws_to_comma_or_rbrace(input) {
            match delim {
                ',' => { /* continue loop to next member */ }
                '}' => {
                    out.emit_char('}')?;
                    break;
                }
                _ => unreachable!(),
            }
        } else {
            // Attempt to preserve trailing '}' when a line comment appears before it
            skip_line_comment_preserving_rbrace(input, opts);
            skip_ws_and_comments(input, opts);
            if input.starts_with('}') {
                *input = &input[1..];
                out.emit_char('}')?;
                break;
            }
            if input.starts_with(',') {
                *input = &input[1..];
            }
        }
    }
    Ok(())
}

fn take_until_delim<'i>(input: &mut &'i str, delims: &[char]) -> &'i str {
    let s = *input;
    let mut end = 0usize;
    for (i, ch) in s.char_indices() {
        if delims.contains(&ch) || ch == '\n' || ch == '\r' {
            break;
        }
        end = i + ch.len_utf8();
    }
    *input = &s[end..];
    &s[..end]
}

#[inline]
fn take_key_until_delim_fast<'i>(input: &mut &'i str) -> Option<&'i str> {
    let s = *input;
    if s.is_empty() {
        return Some("");
    }
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            b' ' | b'\t' | b'\n' | b'\r' | b',' | b'{' | b'}' | b'[' | b']' | b'(' | b')'
            | b':' | b'"' | b'\'' => break,
            b'/' => {
                if i + 1 < b.len() && (b[i + 1] == b'/' || b[i + 1] == b'*') {
                    break;
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    let key = &s[..i];
    *input = &s[i..];
    Some(key)
}

// Fast-skip: if the input slice consists of ASCII whitespace followed by '}',
// consume through '}' and return it. Otherwise, return None without consuming.
#[inline]
fn fast_ws_to_only_rbrace(input: &mut &str) -> Option<char> {
    let s = *input;
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    if let Some(pos) = memchr2(b',', b'}', bytes) {
        // If we found a comma before '}', not an immediate close
        if bytes[pos] == b',' {
            return None;
        }
        // Ensure all bytes before '}' are ASCII whitespace only
        for &b in &bytes[..pos] {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => {}
                _ => return None,
            }
        }
        *input = &s[pos + 1..];
        Some('}')
    } else {
        None
    }
}

// Fast-skip: if input has only ASCII whitespace up to next ',' or '}',
// consume through the delimiter and return it. Otherwise, return None.
#[inline]
fn fast_ws_to_comma_or_rbrace(input: &mut &str) -> Option<char> {
    let s = *input;
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    if let Some(pos) = memchr2(b',', b'}', bytes) {
        // Ensure the prefix is only ASCII whitespace
        for &b in &bytes[..pos] {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => {}
                _ => return None,
            }
        }
        let delim = bytes[pos] as char;
        *input = &s[pos + 1..];
        Some(delim)
    } else {
        None
    }
}
