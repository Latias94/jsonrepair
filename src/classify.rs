#[inline]
pub fn is_whitespace(c: char) -> bool {
    // Include U+FEFF (BOM) as whitespace-equivalent so it can be skipped at root in streaming.
    matches!(
        c,
        '\u{0009}' | '\u{000A}' | '\u{000D}' | '\u{0020}' | '\u{FEFF}'
    )
}

#[inline]
pub fn is_double_quote_like(c: char) -> bool {
    c == '"' || c == '\u{201C}' || c == '\u{201D}'
}

#[inline]
pub fn is_single_quote_like(c: char) -> bool {
    matches!(c, '\u{27}' | '\u{2018}' | '\u{2019}' | '\u{60}' | '\u{B4}')
}
