use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn numbers_leading_dot_in_object() {
    let s = "{a:.25}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], serde_json::json!(0.25));
}

#[test]
fn numbers_trailing_dot_in_object() {
    let s = "{a:1.}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], serde_json::json!(1.0));
}

#[test]
fn numbers_incomplete_exponent_variants() {
    for s in ["{a:1e}", "{a:1E}", "{a:1e+}", "{a:1e-}"] {
        let out = crate::repair_to_string(s, &opts()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["a"], 1);
    }
}

#[test]
fn numbers_quote_suspicious_slash() {
    let s = "{n:1/3}"; // suspicious numeric-like token
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "1/3");
}

#[test]
fn numbers_quote_suspicious_multi_dot() {
    let s = "{n:1.1.1}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "1.1.1");
}

#[test]
fn numbers_quote_suspicious_hyphen_range() {
    let s = "{n:10-20}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "10-20");
}

#[test]
fn numbers_quote_suspicious_word_suffix() {
    let s = "{n:2notanumber}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "2notanumber");
}

#[test]
fn ensure_ascii_escapes_unicode() {
    let mut o = opts();
    o.ensure_ascii = true;
    let out = crate::repair_to_string("{a:'你好', b:'x'}", &o).unwrap();
    // Output must be valid JSON and contain only ASCII
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["b"], "x");
    assert!(!out.chars().any(|c| (c as u32) > 0x7f));
}

#[test]
fn word_comment_markers_before_keys() {
    let mut o = opts();
    o.word_comment_markers = vec!["COMMENT".to_string(), "SHOULD_NOT_EXIST".to_string()];
    let s = "{ 'a':1, COMMENT 'b':2, SHOULD_NOT_EXIST 'c':3 }";
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1, "b":2, "c":3}));
}

#[test]
fn comments_unicode_adjacent_delims() {
    let s = "{'汉'/*注释*/:/*c*/'字'}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["汉"], "字");
}

#[test]
fn array_missing_commas_with_unicode_and_comments() {
    let s = "[ '你' /*c*/ '好' ]"; // missing comma
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!(["你", "好"]));
}

#[test]
fn jsonp_and_fenced_mixed() {
    // Use streaming which tolerates JSONP + fenced with spaces more robustly
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["cb ", "( ```json\n", "{a: '你' + '好'}", "\n``` ) ;\n"];
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":"你好"}));
}

#[test]
fn ndjson_with_hash_comments_and_blanks() {
    let s = "# head\n{a:1}\n\n# mid\n{b:2}\n";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([{"a":1},{"b":2}]));
}

#[test]
fn aggressive_truncation_simple_object() {
    let mut o = opts();
    o.aggressive_truncation_fix = true;
    let s = "{a:1, b: 2, c: {d:3"; // truncated
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], 1);
    assert_eq!(v["b"], 2);
    // c may or may not be fully closed depending on heuristic; ensure object shape
    assert!(v.get("c").is_some());
}

#[test]
fn regex_literal_with_flags_in_object() {
    let s = "{r:/ab+/i}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let r = v.get("r").and_then(|x| x.as_str()).unwrap();
    assert!(r == "/ab+/i" || r == "/ab+/");
}

#[test]
fn concat_strings_with_comments_and_unicode() {
    let s = "'你' /*x*/ + // y\n '好'";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    assert_eq!(out, "\"你好\"");
}

#[test]
fn nonfinite_numbers_normalized_in_array() {
    let s = "[NaN, Infinity, -Infinity, 1]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([null, null, null, 1]));
}

#[test]
fn undefined_in_array_becomes_null() {
    let s = "[undefined, 1]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([null, 1]));
}

#[test]
fn object_missing_colon_with_unicode_and_comment() {
    let s = "{ '键' /*c*/ '值' }"; // missing colon
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"键":"值"}));
}

#[test]
fn object_missing_commas_two_pairs() {
    let s = "{ a:1 /*c*/ b:2 }"; // missing comma, with comment boundary
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1,"b":2}));
}
