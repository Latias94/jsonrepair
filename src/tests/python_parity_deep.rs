use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn strings_unicode_escape_in_single_quotes() {
    let s = "{s:'\\u4f60\\u597d'}"; // '你好'
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["s"], "你好");
}

#[test]
fn strings_control_chars_in_single_quotes() {
    let s = "{t:'a\\tb\\nc'}"; // tab and newline escapes
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["t"], "a\tb\nc");
}

#[test]
fn ensure_ascii_with_emoji() {
    let mut o = opts();
    o.ensure_ascii = true;
    let out = crate::repair_to_string("{e:'😊'}", &o).unwrap();
    assert!(!out.chars().any(|c| (c as u32) > 0x7f));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["e"], "😊");
}

#[test]
fn deep_nested_mixed_missing_and_comments() {
    // { a: [ { b:1 /*c*/ c:2 }, { d:'你' /*c*/ e:'好' } ] }
    let s = "{ a: [ { b:1 /*c*/ c:2 }, { d:'你' /*c*/ e:'好' } ] }";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"][0]["b"], 1);
    assert_eq!(v["a"][0]["c"], 2);
    assert_eq!(v["a"][1]["d"], "你");
    assert_eq!(v["a"][1]["e"], "好");
}

#[test]
fn ndjson_thousand_lines() {
    let mut s = String::new();
    for i in 0..1000 {
        s.push_str(&format!("{{x:{}}}\n", i));
    }
    let out = crate::repair_to_string(&s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v.as_array().map(|a| a.len()), Some(1000));
    assert_eq!(v[0]["x"], 0);
    assert_eq!(v[999]["x"], 999);
}

#[test]
fn ndjson_mixed_comments_and_crlf() {
    let s = "# head\r\n{a:1}//t\r\n\r\n/*m*/{b:2}\r\n";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([{"a":1},{"b":2}]));
}

#[test]
fn fenced_uppercase_json_language() {
    let s = "```JSON\n{a:1}\n```";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn jsonp_non_streaming_simple() {
    let s = "cb({z:3});";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"z":3}));
}

#[test]
fn ellipsis_in_array_only() {
    let s = "{a:[1,2,...,3], b:{x:1, y:2}}"; // ellipsis supported in arrays
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], serde_json::json!([1, 2, 3]));
    assert_eq!(v["b"], serde_json::json!({"x":1, "y":2}));
}

#[test]
fn regex_with_escaped_slash_and_flag() {
    let s = r"{r:/a\/b/i}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let r = v.get("r").and_then(|x| x.as_str()).unwrap();
    // Forward slash escaping is optional in JSON strings; accept both
    assert!(r == "/a/b/i" || r == "/a/b/");
}

#[test]
fn writer_streaming_equivalence_object_with_comments() {
    let s = "{x:1, /*c*/ arr: [1,2, /*x*/ 3], note: '你' + '好'}";
    let o = opts();
    let expect = crate::repair_to_string(s, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(s, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    assert_eq!(expect, got);
}

#[test]
fn aggressive_truncation_in_array() {
    let mut o = opts();
    o.aggressive_truncation_fix = true;
    let s = "[1, 2, 3"; // truncated
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    // Expect the three elements to be retained
    assert_eq!(v, serde_json::json!([1, 2, 3]));
}

#[test]
fn streaming_unicode_near_comment_markers() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let input = "{a:'你'/*注释*/,b:'好'//行注释\n,c:3}";
    let sizes = super::lcg_sizes(13579, input.chars().count());
    let parts = super::chunk_by_char(input, &sizes);
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
    assert_eq!(v, serde_json::json!({"a":"你","b":"好","c":3}));
}

#[test]
fn word_comment_multiple_positions() {
    let mut o = opts();
    o.word_comment_markers = vec!["COMMENT".to_string(), "SKIPME".to_string()];
    let s = "{ COMMENT 'a':1, b:2, SKIPME 'c':3 }";
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1, "b":2, "c":3}));
}

#[test]
fn numbers_incomplete_exponent_in_array() {
    let s = "[1e,2E+,3e-,4]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2, 3, 4]));
}

#[test]
fn array_many_spaces_and_newlines_inside_container() {
    let mut s = String::from("[");
    for i in 0..50 {
        if i > 0 {
            s.push_str(",\n\n    \n\t  ");
        }
        s.push_str(&format!("{}", i));
    }
    s.push(']');
    let out = crate::repair_to_string(&s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 50);
}

#[test]
fn object_many_newlines_and_comments_between_members() {
    let mut s = String::from("{");
    for i in 0..30 {
        if i > 0 {
            s.push_str(",\n//x\n/*y*/\n\n");
        }
        s.push_str(&format!("k{}:{}", i, i));
    }
    s.push('}');
    let out = crate::repair_to_string(&s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v.as_object().unwrap().len(), 30);
}

#[test]
fn streaming_concat_strings_and_regex_mixed() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let input = "{s:'he'+'llo', r:/ab\\/c+/i}\n{t:'你'+'好'}\n";
    let sizes = super::lcg_sizes(24680, input.chars().count());
    let parts = super::chunk_by_char(input, &sizes);
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
    assert_eq!(outs.len(), 2);
    let v1: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&outs[1]).unwrap();
    assert_eq!(v1.get("s").and_then(|x| x.as_str()), Some("hello"));
    assert!(v1.get("r").and_then(|x| x.as_str()).is_some());
    assert_eq!(v2, serde_json::json!({"t":"你好"}));
}
