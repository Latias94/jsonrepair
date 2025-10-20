use crate::options::Options;
use memchr::{memchr, memchr2};

pub fn skip_bom(input: &mut &str) {
    if let Some(rest) = input.strip_prefix('\u{FEFF}') {
        *input = rest;
    }
}

/// Optimized combined whitespace and comment skipper.
/// 🟢 Uses memchr for fast comment scanning while maintaining fast ASCII whitespace path.
#[inline]
pub fn skip_ws_and_comments(input: &mut &str, opts: &Options) {
    loop {
        let before_len = input.len();

        // Fast ASCII whitespace scan using byte-level operations
        let s = *input;
        let bytes = s.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b' ' | b'\t' | b'\n' | b'\r' => i += 1,
                _ => break,
            }
        }
        *input = &s[i..];

        if input.is_empty() {
            break;
        }

        // Line comment: //
        if input.as_bytes().starts_with(b"//") {
            let rest = &input[2..];
            let bytes = rest.as_bytes();
            // 🟢 Use memchr to quickly find newline
            if let Some(pos) = memchr2(b'\n', b'\r', bytes) {
                *input = &rest[pos + 1..];
            } else {
                *input = "";
            }
            continue;
        }

        // Block comment: /* */
        if input.as_bytes().starts_with(b"/*") {
            let rest = &input[2..];
            let bytes = rest.as_bytes();
            // 🟢 Use memchr to quickly find '*', then check for '/'
            let mut off = 0usize;
            let mut closed = false;
            while let Some(p) = memchr(b'*', &bytes[off..]) {
                let idx = off + p;
                if idx + 1 < bytes.len() && bytes[idx + 1] == b'/' {
                    *input = &rest[idx + 2..];
                    closed = true;
                    break;
                }
                off = idx + 1;
            }
            if !closed {
                *input = "";
            }
            continue;
        }

        // Hash comment: #
        if opts.tolerate_hash_comments && input.as_bytes().first() == Some(&b'#') {
            let rest = &input[1..];
            let bytes = rest.as_bytes();
            // 🟢 Use memchr to quickly find newline
            if let Some(pos) = memchr2(b'\n', b'\r', bytes) {
                *input = &rest[pos + 1..];
            } else {
                *input = "";
            }
            continue;
        }

        // No progress made, exit
        if before_len == input.len() {
            break;
        }
    }
}

pub fn starts_with_ident(s: &str) -> bool {
    matches!(s.chars().next(), Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$')
}

pub fn take_ident(s: &str) -> (&str, &str) {
    let mut end = 0usize;
    for (i, ch) in s.char_indices() {
        if i == 0 {
            if !(ch.is_ascii_alphabetic() || ch == '_' || ch == '$') {
                break;
            }
            end = i + ch.len_utf8();
        } else {
            if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') {
                break;
            }
            end = i + ch.len_utf8();
        }
    }
    (&s[..end], &s[end..])
}

/// Take a non-ASCII-friendly symbol token until a delimiter.
/// Delimiters: whitespace or one of , [ ] { } ( ) : ' ".
/// A slash '/' stops only if it starts a comment (// or /*).
pub fn take_symbol_until_delim<'i>(input: &mut &'i str) -> &'i str {
    let s = *input;
    if s.is_empty() {
        return s;
    }
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            // ASCII whitespace or structural delimiters terminate the token
            b' ' | b'\t' | b'\n' | b'\r' | b',' | b'[' | b']' | b'{' | b'}' | b'(' | b')'
            | b':' | b'"' | b'\'' => break,
            b'/' => {
                // Stop only if a comment starts
                if i + 1 < b.len() && (b[i + 1] == b'/' || b[i + 1] == b'*') {
                    break;
                }
                i += 1;
            }
            _ => {
                // Any non-ASCII or regular ASCII continues
                i += 1;
            }
        }
    }
    let tok = &s[..i];
    *input = &s[i..];
    tok
}

/// Skip known "word markers" (e.g., COMMENT) only when present at the
/// current cursor. This function first trims ASCII whitespace, then performs
/// a direct prefix check for each marker. It avoids entering heavier paths
/// when the first character cannot possibly match any marker.
pub fn skip_word_markers(input: &mut &str, markers: &[String]) {
    if markers.is_empty() {
        return;
    }
    loop {
        let before = *input;
        // Fast ASCII whitespace trim
        let s0 = *input;
        let bytes = s0.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b' ' | b'\t' | b'\n' | b'\r' => i += 1,
                _ => break,
            }
        }
        *input = &s0[i..];
        let s1 = *input;
        // Quick pre-filter on first char
        let mut skipped = false;
        if let Some(_c0) = s1.chars().next() {
            for m in markers {
                if s1.starts_with(m.as_str()) {
                    *input = &s1[m.len()..];
                    skipped = true;
                    break;
                }
            }
        }
        if !skipped {
            *input = before;
            break;
        }
    }
}

pub fn skip_ellipsis(input: &mut &str) -> bool {
    if let Some(rest) = input.strip_prefix("...") {
        *input = rest;
        true
    } else {
        false
    }
}

/// If `s` starts with optional ASCII whitespace, followed by an identifier, optional ASCII
/// whitespace, and an opening parenthesis '(', return the byte offset to just after '('.
/// Otherwise return None.
pub fn jsonp_prefix_len(s: &str) -> Option<usize> {
    // Skip ASCII whitespace fast
    let mut idx = 0usize;
    let bytes = s.as_bytes();
    while idx < bytes.len() {
        match bytes[idx] {
            b' ' | b'\t' | b'\n' | b'\r' => idx += 1,
            _ => break,
        }
    }
    let after_ws = &s[idx..];
    if !starts_with_ident(after_ws) {
        return None;
    }
    let (_ident, rest) = take_ident(after_ws);
    let mut off = s.len() - rest.len();
    // Skip ASCII whitespace again
    while off < s.len() {
        match s.as_bytes()[off] {
            b' ' | b'\t' | b'\n' | b'\r' => off += 1,
            _ => break,
        }
    }
    if off < s.len() && s.as_bytes()[off] == b'(' {
        Some(off + 1)
    } else {
        None
    }
}

/// Compute how many bytes to consume after an opening fenced marker ```.
/// Accepts optional language token (ASCII letters/digits/underscore), optional ASCII spaces/tabs,
/// and optional single newline (\n or \r). Returns the number of bytes to skip from the given slice.
/// The input `s` must start immediately after the three backticks.
pub fn fence_open_lang_newline_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    // skip any extra backticks beyond the initial opener ``` to tolerate fences like ````
    while i < bytes.len() && bytes[i] == b'`' {
        i += 1;
    }
    // optional language token: [A-Za-z0-9_]+
    while i < bytes.len() {
        let b = bytes[i];
        let is_lang = b.is_ascii_alphanumeric() || b == b'_';
        if is_lang {
            i += 1;
        } else {
            break;
        }
    }
    // optional spaces/tabs
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' => i += 1,
            _ => break,
        }
    }
    // optional newline (\n or \r)
    if i < bytes.len() && (bytes[i] == b'\n' || bytes[i] == b'\r') {
        i += 1;
    }
    i
}
