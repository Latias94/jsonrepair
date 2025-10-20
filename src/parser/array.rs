use crate::emit::{Emitter, JRResult};
use crate::options::Options;
use super::lex::{skip_ws_and_comments, skip_word_markers, skip_ellipsis};
use memchr::memchr2;
use crate::parser::parse_regex_literal;
use crate::parser::parse_symbol_or_unquoted_string;
use super::strings::parse_string_literal_concat_fast;
use super::number::parse_number_token;

pub fn parse_array<'i, E: Emitter>(input: &mut &'i str, opts: &Options, out: &mut E, logger: &mut crate::parser::Logger) -> JRResult<()> {
    if !input.starts_with('[') { return Ok(()); }
    *input = &input[1..];
    out.emit_char('[')?;
    // Enter-array fast path: if only ASCII whitespace before a closing ']', close immediately.
    if let Some(']') = fast_ws_to_only_rbracket(input) {
        out.emit_char(']')?;
        return Ok(());
    }
    skip_ws_and_comments(input, opts);
    let mut first = true;
    let mut idx = 0usize;
    'outer: loop {
        skip_ws_and_comments(input, opts);
        if input.is_empty() {
            // best-effort close
            out.emit_char(']')?;
            break;
        }
        if input.starts_with(']') {
            *input = &input[1..];
            out.emit_char(']')?;
            break;
        }
        if input.is_empty() { break; }
        skip_word_markers(input, &opts.word_comment_markers);
        while skip_ellipsis(input) { skip_ws_and_comments(input, opts); }
        // optional comma between elements (fast path: ASCII ws -> ',' or ']')
        if let Some(delim) = fast_ws_to_comma_or_rbracket(input) {
            match delim {
                ',' => { /* consumed comma, proceed to parse next element */ }
                ']' => { out.emit_char(']')?; break; }
                _ => unreachable!(),
            }
        } else {
            // After top-of-loop skipper, simply consume a stray comma or close if present
            if input.starts_with(',') { *input = &input[1..]; }
            if input.starts_with(']') { *input = &input[1..]; out.emit_char(']')?; break; }
        }
        // emit comma only when we are going to output an element
        if !first { out.emit_char(',')?; }
        first = false;
        // Track array index for value path
        logger.push_index(idx);
        let c = input.chars().next().unwrap();
        match c {
            '{' => super::object::parse_object(input, opts, out, logger)?,
            '[' => parse_array(input, opts, out, logger)?,
            '"' | '\'' => parse_string_literal_concat_fast(input, opts, out)?,
            '/' => parse_regex_literal(input, opts, out)?,
            c if c == '-' || c == '.' || c.is_ascii_digit() => parse_number_token(input, opts, out)?,
            _ => parse_symbol_or_unquoted_string(input, opts, out, logger)?,
        }
        logger.pop_index();
        idx += 1;
        // Fast path after element: ASCII ws -> next delimiter ',' or ']'
        if let Some(delim) = fast_ws_to_comma_or_rbracket(input) {
            match delim {
                ',' => { continue 'outer; }
                ']' => { out.emit_char(']')?; break 'outer; }
                _ => unreachable!(),
            }
        } else {
            // Fallback: generic skipping and optional comma consumption
            skip_ws_and_comments(input, opts);
            if input.starts_with(',') { *input = &input[1..]; }
        }
    }
    Ok(())
}

#[inline]
fn fast_ws_to_only_rbracket(input: &mut &str) -> Option<char> {
    let s = *input;
    if s.is_empty() { return None; }
    let bytes = s.as_bytes();
    if let Some(pos) = memchr2(b',', b']', bytes) {
        // If we found a comma before ']', this isn't an immediate close; ignore.
        if bytes[pos] == b',' { return None; }
        // Ensure all bytes before ']' are ASCII whitespace only.
        for &b in &bytes[..pos] {
            match b { b' ' | b'\t' | b'\n' | b'\r' => {}, _ => return None }
        }
        *input = &s[pos+1..];
        Some(']')
    } else {
        None
    }
}
#[inline]
fn fast_ws_to_comma_or_rbracket(input: &mut &str) -> Option<char> {
    let s = *input;
    if s.is_empty() { return None; }
    let bytes = s.as_bytes();
    // Find next ',' or ']' quickly
    if let Some(pos) = memchr2(b',', b']', bytes) {
        // Ensure the prefix is only ASCII whitespace
        for &b in &bytes[..pos] {
            match b { b' ' | b'\t' | b'\n' | b'\r' => {}, _ => return None }
        }
        let delim = bytes[pos] as char;
        *input = &s[pos+1..];
        Some(delim)
    } else {
        // No delimiter ahead; nothing to do
        None
    }
}
