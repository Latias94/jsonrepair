use crate::error::{RepairError, RepairErrorKind};
use crate::options::Options;
mod scanner_bytes;
use std::io::Write;

// A lightweight, llm_json-inspired parser using Vec<char> + index scanning.
// Focuses on practical repairs: unquoted keys/strings, trailing/missing commas,
// simple comments, incomplete containers, and string escaping.

pub(crate) fn repair_to_string_impl(input: &str, opts: &Options) -> Result<String, RepairError> {
    // Fast path: valid JSON as-is
    #[cfg(feature = "serde")]
    {
        if !opts.ensure_ascii && opts.assume_valid_json_fastpath {
            return Ok(input.to_string());
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(input) {
            if !opts.ensure_ascii {
                return Ok(serde_json::to_string(&val)
                    .map_err(|e| RepairError::from_serde("serialize", e))?);
            } else {
                use serde::Serialize;
                let mut buf: Vec<u8> = Vec::with_capacity(input.len());
                let mut ser = serde_json::Serializer::with_formatter(&mut buf, AsciiEscaper);
                val.serialize(&mut ser)
                    .map_err(|e| RepairError::from_serde("serialize", e))?;
                return String::from_utf8(buf).map_err(|e| {
                    RepairError::new(RepairErrorKind::Parse(format!("utf8 error: {}", e)), 0)
                });
            }
        }
    }

    // Preprocess: BOM, fenced code blocks (multi-block aggregation / single-block extract), JSONP trimming
    let sfull = skip_bom(input);
    if opts.fenced_code_blocks && sfull.contains("```") {
        let bodies = collect_fenced_bodies(sfull);
        if bodies.len() >= 2 {
            // Multi-block aggregation: parse each fenced body as a JSON value and combine into an array
            let mut agg = String::with_capacity(sfull.len().saturating_add(16));
            agg.push('[');
            for (i, b) in bodies.iter().enumerate() {
                if i > 0 {
                    agg.push(',');
                }
                let mut inner = *b;
                // JSONP unwrap on body level (best-effort)
                while let Some(inner2) = trim_jsonp(inner) {
                    inner = inner2;
                }
                let mut p = LlmCompatParser::new(inner, opts);
                p.parse()?;
                agg.push_str(&p.into_output());
            }
            agg.push(']');
            let out = if opts.python_style_separators {
                apply_python_separators(&agg)
            } else {
                agg
            };
            return Ok(out);
        } else if bodies.len() == 1 {
            let mut inner = bodies[0];
            while let Some(inner2) = trim_jsonp(inner) {
                inner = inner2;
            }
            let mut p = LlmCompatParser::new(inner, opts);
            p.parse()?;
            let mut out = p.into_output();
            if opts.python_style_separators {
                out = apply_python_separators(&out);
            }
            return Ok(out);
        }
        // bodies.len()==0: fall through to normal path (possibly unmatched backticks)
    }

    // Normal path: after BOM removal with optional JSONP wrapper
    let mut s = sfull;
    while let Some(inner) = trim_jsonp(s) {
        s = inner;
    }

    let mut p = LlmCompatParser::new(s, opts);
    p.parse()?;
    let mut out = p.into_output();

    if opts.python_style_separators {
        out = apply_python_separators(&out);
    }
    Ok(out)
}

pub(crate) fn repair_to_writer_impl<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    let s = repair_to_string_impl(input, opts)?;
    writer
        .write_all(s.as_bytes())
        .map_err(|e| RepairError::new(RepairErrorKind::Parse(format!("write error: {}", e)), 0))
}

// Simple pre-trim similar to llm_json behavior:
// - If a ```json ... ``` fenced block exists, extract the inner.
// - Otherwise, leave input unchanged.
// - Also skip an initial UTF-8 BOM if present.
// Kept for clarity: only BOM skipping is relevant now (fenced handling is done above)
fn _pre_trim_llm_only_bom(input: &str) -> &str {
    skip_bom(input)
}

fn skip_bom(s: &str) -> &str {
    const BOM: char = '\u{FEFF}';
    if s.chars().next() == Some(BOM) {
        let mut it = s.char_indices();
        it.next();
        if let Some((pos, _)) = it.next() {
            &s[pos..]
        } else {
            ""
        }
    } else {
        s
    }
}

fn collect_fenced_bodies(s: &str) -> Vec<&str> {
    let mut bodies = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = s[pos..].find("```") {
        let start = pos + rel;
        let after_ticks = start + 3;
        // Skip the first line (language tag etc.) until newline
        let mut body_start = after_ticks;
        if let Some(nl_rel) = s[after_ticks..].find(['\n', '\r']) {
            let nl_abs = after_ticks + nl_rel;
            // Handle CRLF
            if s[nl_abs..].starts_with("\r\n") {
                body_start = nl_abs + 2;
            } else {
                body_start = nl_abs + 1;
            }
        }
        // Find the closing ```
        if let Some(end_rel) = s[body_start..].find("```") {
            let body_end = body_start + end_rel;
            bodies.push(&s[body_start..body_end]);
            pos = body_end + 3;
        } else {
            break;
        }
    }
    bodies
}

fn trim_jsonp(s: &str) -> Option<&str> {
    let rest = s.trim_start();
    // Detect `<ident>( ... )[;]`
    let mut chars = rest.chars();
    let c0 = chars.next()?;
    if !(c0.is_alphabetic() || c0 == '_') {
        return None;
    }
    // Read the ident
    let mut i = c0.len_utf8();
    for ch in rest[i..].chars() {
        if ch.is_alphanumeric() || ch == '_' {
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    let after_ident = rest[i..].trim_start();
    if !after_ident.starts_with('(') {
        return None;
    }
    // 找到最后一个 ')'，取中间体
    if let Some(rpos) = after_ident.rfind(')') {
        let inner = &after_ident[1..rpos];
        Some(inner)
    } else {
        None
    }
}

struct LlmCompatParser<'a> {
    orig: &'a str,
    input: Vec<char>,
    pos: usize,
    out: String,
    ensure_ascii: bool,
    _opts: &'a Options,
    char_to_byte: Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Ctx {
    Root,
    Object,
    Array,
}

impl<'a> LlmCompatParser<'a> {
    fn new(s: &'a str, opts: &'a Options) -> Self {
        let mut char_to_byte = Vec::new();
        for (idx, _) in s.char_indices() {
            char_to_byte.push(idx);
        }
        char_to_byte.push(s.len());
        Self {
            orig: s,
            input: s.chars().collect(),
            pos: 0,
            out: String::with_capacity(s.len().saturating_add(8)),
            ensure_ascii: opts.ensure_ascii,
            _opts: opts,
            char_to_byte,
        }
    }

    fn into_output(self) -> String {
        self.out
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }
    fn skip_comments(&mut self) {
        loop {
            if self.current() == Some('/') {
                if self.input.get(self.pos + 1) == Some(&'/') {
                    self.pos += 2;
                    while let Some(ch) = self.current() {
                        self.pos += 1;
                        if ch == '\n' || ch == '\r' {
                            break;
                        }
                    }
                    continue;
                } else if self.input.get(self.pos + 1) == Some(&'*') {
                    self.pos += 2;
                    while self.pos + 1 < self.input.len() {
                        if self.input[self.pos] == '*' && self.input[self.pos + 1] == '/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
            }
            // optional hash comments
            if self.current() == Some('#') {
                self.pos += 1;
                while let Some(ch) = self.current() {
                    self.pos += 1;
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                }
                continue;
            }
            break;
        }
    }

    fn parse(&mut self) -> Result<(), RepairError> {
        // Best-effort: skip non-JSON preface until likely start
        while let Some(ch) = self.current() {
            if matches!(ch, '{' | '[' | '"' | '\'' | '-')
                || ch.is_ascii_digit()
                || ch.is_alphabetic()
            {
                break;
            }
            self.pos += 1;
        }

        self.parse_value(Ctx::Root)?;
        Ok(())
    }

    fn parse_value(&mut self, ctx: Ctx) -> Result<(), RepairError> {
        self.skip_ws();
        self.skip_comments();
        self.skip_ws();
        match self.current() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') | Some('\'') => self.parse_string_concat(),
            Some('/') => self.parse_regex_literal(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some('.') if self._opts.number_tolerance_leading_dot => self.parse_number(),
            Some(c) if is_ident_start(c) => self.parse_ident_or_literal(ctx),
            Some(c) => {
                // Fallback: treat symbol run as a string
                if c == '}' || c == ']' || c == ',' {
                    // missing value; emit empty string
                    self.out.push('"');
                    self.out.push('"');
                    Ok(())
                } else {
                    self.parse_unquoted_string()
                }
            }
            None => Ok(()),
        }
    }

    fn parse_object(&mut self) -> Result<(), RepairError> {
        self.out.push('{');
        self.pos += 1; // skip '{'

        // Semantics: `expecting_key = true` means the next token should be a key
        let mut expecting_key = true;
        let mut need_comma = false;

        loop {
            let checkpoint = self.pos;
            self.skip_ws();
            self.skip_comments();
            self.skip_ws();

            match self.current() {
                None => {
                    // Incomplete object: close with `}`
                    self.out.push('}');
                    break;
                }
                Some('}') => {
                    self.pos += 1;
                    self.out.push('}');
                    break;
                }
                Some(',') => {
                    // Redundant comma: consume and continue; still expecting a key
                    self.pos += 1;
                    expecting_key = true;
                    continue;
                }
                _ => {
                    // If a previous key:value has just finished but no comma present, synthesize one
                    if need_comma {
                        self.out.push(',');
                    }
                    // Read key
                    if !expecting_key {
                        // 漏掉了逗号或状态错位，恢复到读取 key 的状态
                        expecting_key = true;
                    }
                    match self.current() {
                        Some('"') | Some('\'') => self.parse_string_concat()?,
                        _ => self.parse_unquoted_key()?,
                    }
                    // Colon: optional; synthesize when missing
                    self.skip_ws();
                    if self.current() == Some(':') {
                        self.pos += 1;
                        self.out.push(':');
                    } else {
                        self.out.push(':');
                    }
                    // Read value
                    self.parse_value(Ctx::Object)?;
                    // Completed a key:value pair; next should be `,` or `}`. If a key starts directly, we add a comma.
                    expecting_key = true;
                    need_comma = true;
                }
            }

            if self.pos == checkpoint && self.pos < self.input.len() {
                // Ensure progress to avoid infinite loops
                self.pos += 1;
            }
        }

        Ok(())
    }

    fn parse_unquoted_key(&mut self) -> Result<(), RepairError> {
        // For object keys: stop at whitespace or structural delimiters to avoid swallowing the value
        self.out.push('"');
        // fast path: copy initial ascii run
        self.copy_ascii_key_run();
        while let Some(ch) = self.current() {
            match ch {
                ':' | '}' | ',' => break,
                _ if ch.is_whitespace() => break,
                '"' => {
                    self.out.push_str("\\\"");
                    self.pos += 1;
                }
                '\\' => {
                    self.out.push_str("\\\\");
                    self.pos += 1;
                }
                _ => {
                    self.append_char(ch);
                    self.pos += 1;
                }
            }
            // attempt next ascii run
            self.copy_ascii_key_run();
        }
        self.out.push('"');
        Ok(())
    }

    fn parse_array(&mut self) -> Result<(), RepairError> {
        self.out.push('[');
        self.pos += 1; // skip '['

        let mut need_comma = false;
        loop {
            self.skip_ws();
            self.skip_comments();
            self.skip_ws();
            match self.current() {
                None => {
                    self.out.push(']');
                    break;
                }
                Some(']') => {
                    self.pos += 1;
                    self.out.push(']');
                    break;
                }
                Some(',') => {
                    // skip redundant commas
                    self.pos += 1;
                    self.skip_ws();
                    if matches!(self.current(), Some(']') | None) {
                        continue;
                    }
                }
                _ => {
                    if need_comma {
                        self.out.push(',');
                    }
                    self.parse_value(Ctx::Array)?;
                    need_comma = true;
                }
            }
        }
        Ok(())
    }

    fn parse_string_concat(&mut self) -> Result<(), RepairError> {
        // 解析一个或多个通过 `+` 相连的字符串字面量，合并成一个 JSON 字符串
        let mut buf = String::new();

        // 内部：读取单个字符串段，结果追加到 buf
        let mut read_segment = |this: &mut Self| {
            let quote = this.current().unwrap_or('"');
            if quote == '"' || quote == '\'' {
                this.pos += 1; // skip opening
            }
            let mut esc = false;
            while let Some(ch) = this.current() {
                this.pos += 1;
                if esc {
                    // 保留转义：以反斜杠+原样字符形式写出
                    buf.push('\\');
                    buf.push(ch);
                    esc = false;
                    continue;
                }
                if ch == '\\' {
                    esc = true;
                    continue;
                }
                if ch == quote {
                    break;
                }
                if quote == '\'' && ch == '"' {
                    // 单引号字符串内的双引号需要 JSON 逃逸
                    buf.push_str("\\\"");
                    continue;
                }
                // fast path: ascii run until special/quote (use byte-level scanner)
                if ch.is_ascii() && ch != '"' && ch != '\\' && ch != quote {
                    buf.push(ch);
                    let bstart = this.char_to_byte[this.pos];
                    let bytes = this.orig.as_bytes();
                    let quote_b = if quote == '"' { b'"' } else { b'\'' };
                    let adv = scanner_bytes::ascii_run_string(&bytes[bstart..], quote_b);
                    if adv > 0 {
                        let s =
                            unsafe { std::str::from_utf8_unchecked(&bytes[bstart..bstart + adv]) };
                        buf.push_str(s);
                        this.pos += adv;
                    }
                    continue;
                }
                this.append_char_to(&mut buf, ch);
            }
        };

        // 第一个段
        read_segment(self);

        // 后续：跳过空白/注释 + `+` + 空白/注释 + 段
        loop {
            let checkpoint = self.pos;
            self.skip_ws();
            self.skip_comments();
            self.skip_ws();
            if self.current() != Some('+') {
                break;
            }
            // consume '+'
            self.pos += 1;
            self.skip_ws();
            self.skip_comments();
            self.skip_ws();
            match self.current() {
                Some('"') | Some('\'') => read_segment(self),
                _ => {
                    // 非字符串，回退并结束拼接
                    self.pos = checkpoint;
                    break;
                }
            }
        }

        // 输出合并后的 JSON 字符串
        self.out.push('"');
        self.out.push_str(&buf);
        self.out.push('"');
        Ok(())
    }

    fn parse_unquoted_string(&mut self) -> Result<(), RepairError> {
        self.out.push('"');
        // fast path initial ascii run
        self.copy_ascii_symbol_run();
        while let Some(ch) = self.current() {
            match ch {
                ',' | '}' | ']' | ':' => break,
                '"' => {
                    self.out.push_str("\\\"");
                    self.pos += 1;
                }
                '\\' => {
                    self.out.push_str("\\\\");
                    self.pos += 1;
                }
                _ if ch.is_whitespace() => {
                    // trailing whitespace before delimiter => stop
                    let mut tp = self.pos + 1;
                    let mut found = false;
                    while let Some(nc) = self.input.get(tp) {
                        if matches!(nc, ',' | '}' | ']' | ':') {
                            found = true;
                            break;
                        }
                        if !nc.is_whitespace() {
                            break;
                        }
                        tp += 1;
                    }
                    if found {
                        break;
                    }
                    self.append_char(ch);
                    self.pos += 1;
                }
                _ => {
                    self.append_char(ch);
                    self.pos += 1;
                }
            }
            // attempt another ascii run
            self.copy_ascii_symbol_run();
        }
        self.out.push('"');
        Ok(())
    }

    fn parse_regex_literal(&mut self) -> Result<(), RepairError> {
        // 读取 /.../flags 并作为 JSON 字符串输出
        // 若解析失败，退化为输出 "/" 作为字符串
        if self.current() != Some('/') {
            self.out.push_str("\"/\"");
            return Ok(());
        }
        let s = &self.input;
        let start = self.pos; // at '/'
        self.pos += 1; // skip '/'
        let mut esc = false;
        let mut body = String::new();
        while self.pos < s.len() {
            let ch = s[self.pos];
            self.pos += 1;
            if esc {
                body.push(ch);
                esc = false;
                continue;
            }
            if ch == '\\' {
                body.push(ch);
                esc = true;
                continue;
            }
            if ch == '/' {
                break;
            }
            body.push(ch);
        }
        if self.pos > s.len() || s.get(self.pos - 1) != Some(&'/') {
            // no closing '/': fallback to "/"
            self.pos = start + 1; // consumed '/' already
            self.out.push_str("\"/\"");
            return Ok(());
        }
        // flags: letters only
        let mut flags = String::new();
        while self.pos < s.len() {
            let ch = s[self.pos];
            if ch.is_ascii_alphabetic() {
                flags.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        // Emit as JSON string
        self.out.push('"');
        self.out.push('/');
        for ch in body.chars() {
            // 只需要确保 JSON 有效：转义反斜杠与双引号，其他按 ensure_ascii 输出
            match ch {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                _ => self.append_char(ch),
            }
        }
        self.out.push('/');
        self.out.push_str(&flags);
        self.out.push('"');
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), RepairError> {
        let opts = self._opts;
        let start = self.pos;

        // 先取一个“数字样式”的连续片段用于可疑性判断与解析
        let mut end_seg = start;
        while end_seg < self.input.len() {
            let ch = self.input[end_seg];
            // 分隔符或空白
            if ch.is_whitespace() || matches!(ch, ',' | '}' | ']' | ')' | '(' | ':') {
                break;
            }
            // 注释起始
            if ch == '/' {
                if end_seg + 1 < self.input.len() {
                    let n2 = self.input[end_seg + 1];
                    if n2 == '/' || n2 == '*' {
                        break;
                    }
                }
            }
            end_seg += 1;
        }
        // 解析合法的数值前缀，按容错策略调整
        let mut i = start;
        let mut buf = String::new();
        let mut started_with_dot = false;
        let mut ends_with_dot = false;

        // 可选符号
        let mut negative = false;
        if i < self.input.len() && self.input[i] == '-' {
            buf.push('-');
            i += 1;
            negative = true;
        }
        // 负号后若紧跟字母，尝试识别 -Infinity/-NaN
        if negative && i < end_seg {
            let c = self.input[i];
            if c.is_alphabetic() || c == '_' {
                // 捕获标识符
                let mut j = i;
                while j < end_seg {
                    let ch = self.input[j];
                    if ch.is_alphanumeric() || ch == '_' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                let word: String = self.input[i..j].iter().collect::<String>().to_lowercase();
                if self._opts.normalize_js_nonfinite && (word == "infinity" || word == "nan") {
                    self.out.push_str("null");
                    self.pos = j; // 消费到单词末尾
                    return Ok(());
                } else {
                    // 视为普通符号：回退并按字符串处理（含负号一起回退）
                    self.pos = start;
                    return self.parse_unquoted_string();
                }
            }
        }
        // 统计可疑模式（放在非有限识别之后，以免 -Infinity 被过早判定为“可疑数字”而无法规范化）
        let mut dot_count = 0usize;
        let mut has_alpha_non_e = false;
        let mut has_slash = false;
        let mut hyphen_suspicious = false;
        let mut prev: Option<char> = None;
        for idx in start..end_seg {
            let ch = self.input[idx];
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
        if opts.number_quote_suspicious
            && (has_alpha_non_e || has_slash || dot_count > 1 || hyphen_suspicious)
        {
            // 将整个片段作为字符串输出
            self.out.push('"');
            for idx in start..end_seg {
                let ch = self.input[idx];
                match ch {
                    '"' => self.out.push_str("\\\""),
                    '\\' => self.out.push_str("\\\\"),
                    _ => self.append_char(ch),
                }
            }
            self.out.push('"');
            self.pos = end_seg;
            return Ok(());
        }
        // 整数或以点开头
        if i < end_seg {
            let ch = self.input[i];
            if ch == '.' {
                started_with_dot = true;
                buf.push('.');
                i += 1;
                let mut any = 0usize;
                while i < end_seg {
                    let c = self.input[i];
                    if c.is_ascii_digit() {
                        buf.push(c);
                        i += 1;
                        any += 1;
                    } else {
                        break;
                    }
                }
                if any == 0 {
                    // '.x' 后无数字，回退为字符串
                    self.pos = end_seg;
                    self.out.push_str("\".\"");
                    return Ok(());
                }
            } else {
                // 整数部分
                while i < end_seg {
                    let c = self.input[i];
                    if c.is_ascii_digit() {
                        buf.push(c);
                        i += 1;
                    } else {
                        break;
                    }
                }
                // 小数
                if i < end_seg && self.input[i] == '.' {
                    buf.push('.');
                    i += 1;
                    let mut anyf = 0usize;
                    while i < end_seg {
                        let c2 = self.input[i];
                        if c2.is_ascii_digit() {
                            buf.push(c2);
                            i += 1;
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
        // 指数
        let mut exp_invalid = false;
        let mut advance_to = 0usize;
        if i < end_seg {
            let c = self.input[i];
            if c == 'e' || c == 'E' {
                let _base_len = buf.len();
                // 如果容忍不完整指数=false，则仍可选择补0，但遵循主实现我们选择“丢弃指数”
                i += 1;
                let mut j = i;
                if j < end_seg && (self.input[j] == '+' || self.input[j] == '-') {
                    j += 1;
                }
                let mut any = 0usize;
                let mut jj = j;
                while jj < end_seg {
                    let c3 = self.input[jj];
                    if c3.is_ascii_digit() {
                        jj += 1;
                        any += 1;
                    } else {
                        break;
                    }
                }
                if any == 0 {
                    if self._opts.number_tolerance_incomplete_exponent {
                        // 丢弃指数，仅前缀为有效数字
                        exp_invalid = true;
                        advance_to = jj; // 消费到指数末尾（含可选符号）
                    // buf 保持 base_len（不包含 e）
                    } else {
                        // 不容忍：整体当作字符串处理
                        self.pos = end_seg;
                        // emit original segment as string
                        self.out.push('"');
                        for idx in start..end_seg {
                            let ch = self.input[idx];
                            match ch {
                                '"' => self.out.push_str("\\\""),
                                '\\' => self.out.push_str("\\\\"),
                                _ => self.append_char(ch),
                            }
                        }
                        self.out.push('"');
                        return Ok(());
                    }
                } else {
                    // 正常指数：写入 'e' 与可选符号及数字（统一为小写 e）
                    buf.push('e');
                    // 写入可选符号
                    if j > i {
                        buf.push(self.input[i]);
                    }
                    // 写入指数数字
                    for k in j..jj {
                        buf.push(self.input[k]);
                    }
                    i = jj;
                }
            }
        }

        // 组成最终 token 与推进位置
        let consumed_end = if exp_invalid && advance_to > i {
            advance_to
        } else {
            i
        };
        self.pos = consumed_end;

        // 如果什么都没解析（或只有'-'），退化为字符串
        if buf.is_empty() || buf == "-" {
            return self.parse_unquoted_string();
        }

        // Leading zeros policy
        if let Some(first) = buf.chars().next() {
            let t = if first == '-' { &buf[1..] } else { &buf[..] };
            if t.len() > 1 && t.as_bytes()[0] == b'0' && t.as_bytes()[1].is_ascii_digit() {
                use crate::options::LeadingZeroPolicy as LZP;
                match opts.leading_zero_policy {
                    LZP::KeepAsNumber => {}
                    LZP::QuoteAsString => {
                        self.out.push('"');
                        for ch in buf.chars() {
                            match ch {
                                '"' => self.out.push_str("\\\""),
                                '\\' => self.out.push_str("\\\\"),
                                _ => self.append_char(ch),
                            }
                        }
                        self.out.push('"');
                        return Ok(());
                    }
                }
            }
        }

        // Leading dot tolerance
        if started_with_dot && opts.number_tolerance_leading_dot {
            if buf.starts_with("-.") {
                let mut fixed = String::from("-0");
                fixed.push_str(&buf[2..]);
                self.out.push_str(&fixed);
                return Ok(());
            } else if buf.starts_with('.') {
                let mut fixed = String::from("0");
                fixed.push_str(&buf[1..]);
                self.out.push_str(&fixed);
                return Ok(());
            }
        }

        // Trailing dot tolerance
        if ends_with_dot && opts.number_tolerance_trailing_dot {
            buf.push('0');
        }

        // 正常输出数字 token
        self.out.push_str(&buf);
        Ok(())
    }

    fn parse_ident_or_literal(&mut self, _ctx: Ctx) -> Result<(), RepairError> {
        // Capture a run of identifier characters
        let start = self.pos;
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let orig: String = self.input[start..self.pos].iter().collect();
        let ident = orig.to_lowercase();

        // JSON canonical keywords (must be lowercase)
        if orig == "true" {
            self.out.push_str("true");
            return Ok(());
        }
        if orig == "false" {
            self.out.push_str("false");
            return Ok(());
        }
        if orig == "null" {
            self.out.push_str("null");
            return Ok(());
        }

        // Python-style keywords (case-insensitive) gated by option
        if self._opts.allow_python_keywords {
            match ident.as_str() {
                "true" => {
                    self.out.push_str("true");
                    return Ok(());
                }
                "false" => {
                    self.out.push_str("false");
                    return Ok(());
                }
                "none" => {
                    self.out.push_str("null");
                    return Ok(());
                }
                _ => {}
            }
        }

        // JavaScript non-finite (NaN/Infinity) gated by option
        if self._opts.normalize_js_nonfinite {
            match ident.as_str() {
                "nan" | "infinity" => {
                    self.out.push_str("null");
                    return Ok(());
                }
                _ => {}
            }
        }

        // undefined -> null (gated by option)
        if ident.as_str() == "undefined" && self._opts.repair_undefined {
            self.out.push_str("null");
            return Ok(());
        }

        // Unquoted symbol -> quote it
        self.pos = start; // rewind and reuse string path to preserve case
        self.parse_unquoted_string()
    }

    fn append_char(&mut self, ch: char) {
        if !self.ensure_ascii || ch.is_ascii() {
            self.out.push(ch);
        } else {
            // Encode as \uXXXX (or surrogate pair when needed)
            let cp = ch as u32;
            if cp <= 0xFFFF {
                use std::fmt::Write as _;
                let _ = write!(&mut self.out, "\\u{:04X}", cp);
            } else {
                let v = cp - 0x10000;
                let high = 0xD800 + ((v >> 10) & 0x3FF);
                let low = 0xDC00 + (v & 0x3FF);
                use std::fmt::Write as _;
                let _ = write!(&mut self.out, "\\u{:04X}\\u{:04X}", high, low);
            }
        }
    }

    fn append_char_to(&self, buf: &mut String, ch: char) {
        if !self.ensure_ascii || ch.is_ascii() {
            // 仅转义 JSON 关键字符在上层调用处理（如 '"' 在单引号分支内）
            buf.push(ch);
        } else {
            let cp = ch as u32;
            if cp <= 0xFFFF {
                use std::fmt::Write as _;
                let _ = write!(buf, "\\u{:04X}", cp);
            } else {
                let v = cp - 0x10000;
                let high = 0xD800 + ((v >> 10) & 0x3FF);
                let low = 0xDC00 + (v & 0x3FF);
                use std::fmt::Write as _;
                let _ = write!(buf, "\\u{:04X}\\u{:04X}", high, low);
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

impl<'a> LlmCompatParser<'a> {
    fn copy_ascii_key_run(&mut self) {
        let mut i = self.pos;
        while let Some(&ch) = self.input.get(i) {
            if !ch.is_ascii() {
                break;
            }
            if ch == ':' || ch == '}' || ch == ',' {
                break;
            }
            if ch.is_whitespace() {
                break;
            }
            if ch == '"' || ch == '\\' {
                break;
            }
            i += 1;
        }
        if i > self.pos {
            for &c in &self.input[self.pos..i] {
                self.out.push(c);
            }
            self.pos = i;
        }
    }

    fn copy_ascii_symbol_run(&mut self) {
        let mut i = self.pos;
        while let Some(&ch) = self.input.get(i) {
            if !ch.is_ascii() {
                break;
            }
            if ch == ',' || ch == '}' || ch == ']' || ch == ':' {
                break;
            }
            if ch.is_whitespace() {
                break;
            }
            if ch == '"' || ch == '\\' {
                break;
            }
            i += 1;
        }
        if i > self.pos {
            for &c in &self.input[self.pos..i] {
                self.out.push(c);
            }
            self.pos = i;
        }
    }
}

// ASCII-only string formatter (serde serializer) for ensure_ascii=true
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
            if ch <= '\u{7F}' {
                continue;
            }
            if i > start {
                writer.write_all(&fragment_bytes[start..i])?;
            }
            let cp = ch as u32;
            if cp <= 0xFFFF {
                write!(writer, "\\u{:04X}", cp)?;
            } else {
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

fn apply_python_separators(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 10);
    let mut in_str = false;
    let mut esc = false;
    let mut quote = '\0';
    for ch in s.chars() {
        if in_str {
            out.push(ch);
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == quote {
                in_str = false;
            }
        } else {
            match ch {
                '"' | '\'' => {
                    in_str = true;
                    quote = ch;
                    out.push(ch);
                }
                ':' | ',' => {
                    out.push(ch);
                    out.push(' ');
                }
                _ => out.push(ch),
            }
        }
    }
    out
}
