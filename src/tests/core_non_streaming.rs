use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn repair_unquoted_key_and_single_quotes() {
    let s = "{'a':2, b: 'x'}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    assert!(out.contains("\"a\""));
    assert!(out.contains("\"b\""));
    assert!(out.contains("\"x\""));
    assert!(out.starts_with('{') && out.ends_with('}'));
}

#[test]
fn repair_missing_colon_and_comma() {
    let s = "{\n  'a' 2  'b' 3\n}"; // both pairs miss ':' and ','
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], 2);
    assert_eq!(v["b"], 3);
}

#[test]
fn repair_array_missing_commas() {
    let s = "[1 2 3]"; // missing commas between elements
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2, 3]));
}

#[test]
fn strip_comments() {
    let s = "{/* block */\n // line //\n # hash\n 'a': 1}"; // comments outside strings must be ignored
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], 1);
}

#[test]
fn python_keywords_and_undefined() {
    let s = "{ok: True, bad: undefined, none: None, nope: False}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["bad"].is_null());
    assert!(v["none"].is_null());
    assert_eq!(v["nope"], false);
}

#[test]
fn repair_unclosed_braces_and_brackets() {
    let s1 = "{ 'a': 1"; // missing right curly
    let s2 = "[1, 2"; // missing right bracket
    let out1 = crate::repair_to_string(s1, &opts()).unwrap();
    let out2 = crate::repair_to_string(s2, &opts()).unwrap();
    serde_json::from_str::<serde_json::Value>(&out1).unwrap();
    serde_json::from_str::<serde_json::Value>(&out2).unwrap();
}

#[test]
fn regex_literal_to_string() {
    let s = "/ab[c]+/";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    assert_eq!(out, "\"/ab[c]+/\"");
}

#[test]
fn function_call_wrapper_strips() {
    let s = "callback({a:2});";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":2}));
}

#[test]
fn concatenated_strings_merge() {
    let s = "\"hello\" + /*c*/ \" world\"";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    assert_eq!(out, "\"hello world\"");
}

#[test]
fn ndjson_wrapping() {
    let s = "{a:1}\n{b:2}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([{"a":1},{"b":2}]));
}

#[test]
fn fenced_code_block_is_skipped() {
    let s = "```json\n{a:1}\n```";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn leading_zero_policy_quote() {
    let mut o = opts();
    o.leading_zero_policy = LeadingZeroPolicy::QuoteAsString;
    let out = crate::repair_to_string("{n:007}", &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "007");
}

#[test]
fn repair_log_reports_changes() {
    let s = "{ok: True, bad: undefined}"; // python keyword + undefined
    let (out, log) = crate::repair_to_string_with_log(s, &opts()).unwrap();
    assert!(log.iter().any(|e| e.message.contains("python")));
    assert!(log.iter().any(|e| e.message.contains("undefined")));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["bad"].is_null());
}
