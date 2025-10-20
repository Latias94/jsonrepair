#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum LeadingZeroPolicy {
    /// Keep numbers with leading zeros as-is (may be non-strict JSON, but pragmatic).
    KeepAsNumber,
    /// Quote numbers with leading zeros as strings, like "007".
    QuoteAsString,
}

#[derive(Clone, Debug)]
pub struct Options {
    /// Treat `#` as a line comment (in addition to // and /* */) when not inside strings.
    pub tolerate_hash_comments: bool,
    /// Convert the JavaScript value `undefined` into `null` when encountered as a value or symbol.
    pub repair_undefined: bool,
    /// Policy for numbers with leading zeros like 007.
    pub leading_zero_policy: LeadingZeroPolicy,
    /// Skip Markdown fenced code block like ```json ... ``` around the JSON.
    pub fenced_code_blocks: bool,
    /// Enable repair logging. Use `repair_to_string_with_log` to retrieve logs.
    pub logging: bool,
    /// Accept and normalize Python-style keywords True/False/None.
    pub allow_python_keywords: bool,
    /// When true, escape non-ASCII characters in strings as \uXXXX.
    pub ensure_ascii: bool,
    /// Assume input is valid JSON on fast path and skip full serde validation when possible.
    /// Only applied when `ensure_ascii == false`. Disabled by default for safety.
    pub assume_valid_json_fastpath: bool,
    /// Context window size used when building log context snippets.
    /// Controls how many characters are captured on both sides of the position.
    pub log_context_window: usize,
    /// When enabled, attach a JSON path to log entries (non-streaming only).
    /// Currently tracks array indices; object keys will be added in a next iteration.
    pub log_json_path: bool,
    /// Normalize JavaScript non-finite numbers (NaN/Infinity/-Infinity) to null.
    /// Enabled by default for pragmatic interoperability.
    pub normalize_js_nonfinite: bool,
    /// Aggregate streaming NDJSON outputs into a single JSON array.
    /// When enabled, `StreamRepairer::push` will buffer values and only emit on `flush()`.
    pub stream_ndjson_aggregate: bool,
    /// Tolerance: treat a leading dot ".25" as "0.25".
    pub number_tolerance_leading_dot: bool,
    /// Tolerance: treat a trailing dot "1." as "1.0".
    pub number_tolerance_trailing_dot: bool,
    /// Tolerance: an incomplete exponent like "1e" falls back to the base number "1".
    pub number_tolerance_incomplete_exponent: bool,
    /// Quote suspicious number-like tokens containing non-number separators (e.g. 1/3, 10-20).
    /// Currently reserved for future use.
    pub number_quote_suspicious: bool,
    /// Compatibility preset: enable Python-friendly tolerance behaviors.
    pub compat_python_friendly: bool,
    /// Optional word comment markers like "COMMENT" that will be stripped
    /// when found in safe positions (e.g., before an object key). Empty by default.
    pub word_comment_markers: Vec<String>,
    /// Aggressive truncation fix: when encountering extreme truncation inside
    /// an object/array, close the container early at a nearby safe boundary
    /// instead of failing or emitting null. Disabled by default.
    pub aggressive_truncation_fix: bool,
    /// Internal: prevent non-streaming parser from delegating to streaming fallback
    /// to avoid recursive delegation when called from StreamRepairer.
    pub(crate) internal_no_stream_fallback: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tolerate_hash_comments: true,
            repair_undefined: true,
            leading_zero_policy: LeadingZeroPolicy::KeepAsNumber,
            fenced_code_blocks: true,
            logging: false,
            allow_python_keywords: true,
            ensure_ascii: false,
            assume_valid_json_fastpath: false,
            log_context_window: 10,
            log_json_path: false,
            normalize_js_nonfinite: true,
            stream_ndjson_aggregate: false,
            number_tolerance_leading_dot: true,
            number_tolerance_trailing_dot: true,
            number_tolerance_incomplete_exponent: true,
            number_quote_suspicious: true,
            compat_python_friendly: false,
            word_comment_markers: Vec::new(),
            aggressive_truncation_fix: false,
            internal_no_stream_fallback: false,
        }
    }
}
