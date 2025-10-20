#![allow(clippy::collapsible_if)]
#![allow(clippy::needless_borrow)]

use crate::emit::{Emitter, JRResult};
use crate::options::{LeadingZeroPolicy, Options};

pub fn parse_number_token<E: Emitter>(
    input: &mut &str,
    opts: &Options,
    out: &mut E,
) -> JRResult<()> {
    let s = *input;
    // JS non-finite special-case handled here as a robust fallback
    if opts.normalize_js_nonfinite && s.starts_with("-Infinity") {
        *input = &s[9..];
        return out.emit_str("null");
    }
    // Work directly on &str; keep operations ASCII-focused where possible
    let mut end_seg = 0usize;
    // Collect a contiguous numeric-like segment until a clear delimiter or comment start.
    while end_seg < s.len() {
        let ch = s[end_seg..].chars().next().unwrap();
        let l = ch.len_utf8();
        // Stop at common delimiters or whitespace
        if ch.is_whitespace() || matches!(ch, ',' | '}' | ']' | ')' | '(' | ':') {
            break;
        }
        // Stop if '/' starts a comment
        if ch == '/' {
            let p = end_seg + l;
            if p < s.len() {
                let mut it = s[p..].chars();
                if let Some(nc) = it.next() {
                    if nc == '*' || nc == '/' {
                        break;
                    }
                }
            }
        }
        // Otherwise, include
        end_seg += l;
    }
    let seg = &s[..end_seg];

    // Quick suspicious checks on the entire segment
    let mut dot_count = 0usize;
    let mut has_alpha_non_e = false;
    let mut has_slash = false;
    let mut hyphen_suspicious = false;
    let mut prev: Option<char> = None;
    for ch in seg.chars() {
        match ch {
            '.' => dot_count += 1,
            'a'..='z' | 'A'..='Z' => {
                if ch != 'e' && ch != 'E' {
                    has_alpha_non_e = true;
                }
            }
            '/' => has_slash = true,
            '-' => {
                if let Some(p) = prev {
                    if p != 'e' && p != 'E' {
                        hyphen_suspicious = true;
                    }
                }
            }
            _ => {}
        }
        prev = Some(ch);
    }
    if has_alpha_non_e || has_slash || dot_count > 1 || hyphen_suspicious {
        *input = &s[end_seg..];
        return crate::parser::strings::emit_json_string_from_lit(out, seg, opts.ensure_ascii);
    }

    // Parse a valid JSON number prefix from the start with tolerances.
    let mut i = 0usize;
    let mut started_with_dot = false;
    let mut ends_with_dot = false;
    // optional sign
    if let Some('-') = s.chars().next() {
        i += 1;
    }
    // integer or leading dot
    if i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        if ch == '.' {
            started_with_dot = true;
            i += 1;
            // fraction digits
            let mut any = 0usize;
            while i < s.len() {
                let c = s[i..].chars().next().unwrap();
                if c.is_ascii_digit() {
                    i += c.len_utf8();
                    any += 1;
                } else {
                    break;
                }
            }
            if any == 0 {
                // no digits after ., fallback to string
                *input = &s[end_seg..];
                return crate::parser::strings::emit_json_string_from_lit(
                    out,
                    seg,
                    opts.ensure_ascii,
                );
            }
        } else {
            // integer digits
            let mut _any = 0usize; // counter not used afterwards
            while i < s.len() {
                let c = s[i..].chars().next().unwrap();
                if c.is_ascii_digit() {
                    i += c.len_utf8();
                    _any += 1;
                } else {
                    break;
                }
            }
            // optional fraction
            if i < s.len() {
                let c = s[i..].chars().next().unwrap();
                if c == '.' {
                    i += 1;
                    let mut anyf = 0usize;
                    while i < s.len() {
                        let c2 = s[i..].chars().next().unwrap();
                        if c2.is_ascii_digit() {
                            i += c2.len_utf8();
                            anyf += 1;
                        } else {
                            break;
                        }
                    }
                    if anyf == 0 {
                        ends_with_dot = true;
                    }
                }
            }
        }
    }
    // optional exponent
    let mut exp_invalid = false;
    let mut advance_to = 0usize; // where input should advance to (can be beyond tok end if we drop invalid exponent)
    if i < s.len() {
        let c = s[i..].chars().next().unwrap();
        if c == 'e' || c == 'E' {
            let base_end = i;
            i += 1; // include 'e'
            // optional sign
            if i < s.len() {
                let c2 = s[i..].chars().next().unwrap();
                if c2 == '+' || c2 == '-' {
                    i += c2.len_utf8();
                }
            }
            // digits
            let mut any = 0usize;
            while i < s.len() {
                let c3 = s[i..].chars().next().unwrap();
                if c3.is_ascii_digit() {
                    i += c3.len_utf8();
                    any += 1;
                } else {
                    break;
                }
            }
            if any == 0 {
                // invalid exponent -> drop exponent entirely (keep base)
                advance_to = i; // advance past 'e' and optional sign
                i = base_end; // but keep token end at base
                exp_invalid = true;
            }
        }
    }

    // Now i is the end of valid numeric prefix within s; do not overrun segment end
    if i > end_seg {
        i = end_seg;
    }
    let tok = &s[..i];
    let consumed_end = if exp_invalid && advance_to > i {
        advance_to
    } else {
        i
    };
    *input = &s[consumed_end..];

    if tok.is_empty() {
        return out.emit_str("0");
    }

    // Leading zeros policy (after optional '-')
    if let Some(first) = tok.chars().next() {
        let t = if first == '-' { &tok[1..] } else { tok };
        if t.len() > 1 && t.as_bytes()[0] == b'0' && t.as_bytes()[1].is_ascii_digit() {
            match opts.leading_zero_policy {
                LeadingZeroPolicy::KeepAsNumber => {}
                LeadingZeroPolicy::QuoteAsString => {
                    return crate::parser::strings::emit_json_string_from_lit(
                        out,
                        &tok,
                        opts.ensure_ascii,
                    );
                }
            }
        }
    }

    // Leading dot tolerance
    if started_with_dot && opts.number_tolerance_leading_dot {
        if let Some(stripped) = tok.strip_prefix('-') {
            let mut buf = String::from("-0");
            buf.push_str(stripped);
            return out.emit_str(&buf);
        } else {
            let mut buf = String::from("0");
            buf.push_str(tok);
            return out.emit_str(&buf);
        }
    }
    // Trailing dot tolerance (only if not a suspicious double-dot case which we handled earlier)
    if ends_with_dot && opts.number_tolerance_trailing_dot {
        let mut buf = String::from(tok);
        buf.push('0');
        return out.emit_str(&buf);
    }

    out.emit_str(tok)
}
