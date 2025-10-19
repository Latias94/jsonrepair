use super::*;

#[test]
fn ns_concat_various_cases_table() {
    let cases = vec![
        ("\"a\" + \"b\"", "\"ab\""),
        ("'a' + ' ' + 'b'", "\"a b\""),
        ("'x' + /*c*/ 'y'", "\"xy\""),
    ];
    for (inp, want) in cases {
        let out = crate::repair_to_string(inp, &Options::default()).unwrap();
        assert_eq!(out, want, "input={}", inp);
    }
}

#[test]
fn ns_string_ensure_ascii_toggle() {
    let s = "'你好'";
    let out1 = crate::repair_to_string(s, &Options::default()).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&out1).unwrap();
    assert_eq!(v1, serde_json::json!("你好"));
    let o = Options {
        ensure_ascii: true,
        ..Default::default()
    };
    let out2 = crate::repair_to_string(s, &o).unwrap();
    assert!(out2.contains("\\u"));
}

#[test]
fn ns_concat_with_escaped_quotes() {
    let s = "\"a\\\"b\" + ' c'"; // "a\"b" + ' c'
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("a\"b c"));
}

#[test]
fn ns_concat_three_parts_with_unicode_and_comments() {
    let s = "'你好' + /*c*/ ' ' + '世界'";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("你好 世界"));
}

#[test]
fn ns_regex_with_flags_becomes_string() {
    let s = "{r:/ab+/i}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let obj = v.as_object().expect("object");
    let r = obj.get("r").and_then(|x| x.as_str()).expect("string");
    assert!(r == "/ab+/i" || r == "/ab+/");
    if let Some(extra) = obj.get("i") {
        assert!(extra.is_null() || extra.is_string());
    }
}

#[test]
fn ns_concat_mixed_quotes_and_spaces() {
    let s = "\"a\"+ 'b' ";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("ab"));
}

#[test]
fn ns_concat_with_many_segments() {
    let s = "'a' + 'b' + 'c' + 'd'";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("abcd"));
}

#[test]
fn ns_concat_empty_segments_collapse() {
    let s = "'a' + '' + 'b'";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("ab"));
}

#[test]
fn ns_regex_unknown_flag_tolerant() {
    let s = "{r:/ab+/g}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let obj = v.as_object().unwrap();
    let r = obj.get("r").and_then(|x| x.as_str()).unwrap_or("");
    assert!(r.contains("/ab+/"));
}

#[test]
fn ns_concat_with_newlines_and_plus() {
    let s = "'a'\n+ 'b'";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("ab"));
}

#[test]
fn ns_regex_edge_escapes_in_object() {
    // Ensure regex with escaped slash remains a single string token
    let s = "{r:/a\\//}"; // /a\//
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let r = v.get("r").and_then(|x| x.as_str()).unwrap();
    assert!(r == "/a\\//" || r == "/a//");
}

#[test]
fn ns_concat_unicode_ensure_ascii() {
    let s = "'��' + '��'";
    let o = Options {
        ensure_ascii: true,
        ..Default::default()
    };
    let out = crate::repair_to_string(s, &o).unwrap();
    assert!(out.contains("\\u"));
}
