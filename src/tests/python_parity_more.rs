use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn numbers_negative_leading_dot() {
    let s = "{a:-.5}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], serde_json::json!(-0.5));
}

#[test]
fn numbers_negative_trailing_dot() {
    let s = "{a:-1.}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], serde_json::json!(-1.0));
}

#[test]
fn numbers_leading_zeros_quote_policy() {
    let mut o = opts();
    o.leading_zero_policy = LeadingZeroPolicy::QuoteAsString;
    let out = crate::repair_to_string("{a:000.125}", &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], "000.125");
}

#[test]
fn strings_nested_quotes_and_escapes() {
    let s = "{t:'He said \"hi\"', u:\"it's ok\"}"; // keep u as JSON string, key unquoted
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["t"], "He said \"hi\"");
    assert_eq!(v["u"], "it's ok");
}

#[test]
fn strings_windows_path_backslashes() {
    let s = "{p:'C:\\\\Users\\\\Me'}"; // two backslashes in JSON after repair
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["p"], "C:\\Users\\Me");
}

#[test]
fn object_missing_colon_and_commas_combo() {
    let s = "{ 'a' 2 'b' 3 'c' 4 }"; // keys quoted: both missing ':' and ',' are fixed
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":2, "b":3, "c":4}));
}

#[test]
fn array_missing_comma_with_comment_boundary() {
    let s = "[1/*x*/2, 3]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2, 3]));
}

#[test]
fn nested_markers_stripped_before_keys() {
    let mut o = opts();
    o.word_comment_markers = vec!["COMMENT".to_string(), "SHOULD_NOT_EXIST".to_string()];
    let s = "{ outer: { COMMENT 'inner': 1, SHOULD_NOT_EXIST 'z': 2 } }"; // quote keys after markers
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["outer"], serde_json::json!({"inner":1, "z":2}));
}

#[test]
fn streaming_concat_strings_cross_chunks() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let input = "['你' + '好', 'a' + 'b']";
    let sizes = super::lcg_sizes(2024, input.chars().count());
    let parts = super::chunk_by_char(input, &sizes);
    let mut outs = Vec::new();
    for p in parts.iter() {
        if let Some(s) = r.push(p).unwrap() {
            outs.push(s);
        }
    }
    if let Some(tail) = r.flush().unwrap() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!(["你好", "ab"]));
}

#[test]
fn streaming_ndjson_aggregate_with_comments_and_blanks() {
    let mut o = opts();
    o.stream_ndjson_aggregate = true;
    let mut r = crate::StreamRepairer::new(o);
    let input = "# c1\n{a:1}\n\n# c2\n{b:2}\n";
    let sizes = super::lcg_sizes(4242, input.chars().count());
    let parts = super::chunk_by_char(input, &sizes);
    for p in parts.iter() {
        let _ = r.push(p).unwrap();
    }
    let s = r.flush().unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!([{"a":1},{"b":2}]));
}

#[test]
fn streaming_jsonp_fenced_unicode_random_chunks() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let input = "cb(```json\n{a:'你'+'好'}\n```);\n";
    let sizes = super::lcg_sizes(7777, input.chars().count());
    let parts = super::chunk_by_char(input, &sizes);
    let mut outs = Vec::new();
    for p in parts.iter() {
        if let Some(s) = r.push(p).unwrap() {
            outs.push(s);
        }
    }
    if let Some(tail) = r.flush().unwrap() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":"你好"}));
}

#[test]
fn writer_streaming_matches_to_string_large_array() {
    // Build a moderately large array
    let mut s = String::from("[");
    for i in 0..500usize {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{k{}:{}}}", i, i));
    }
    s.push(']');
    let o = opts();
    let expect = crate::repair_to_string(&s, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(&s, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&expect).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&got).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn logging_path_array_index_for_undefined() {
    let mut o = opts();
    o.logging = true;
    o.log_json_path = true;
    o.log_context_window = 8;
    let input = "[0, undefined, 2]";
    let (_out, log) = crate::repair_to_string_with_log(input, &o).unwrap();
    let mut saw = false;
    for e in log {
        if e.message == "replaced undefined with null" && e.path.as_deref() == Some("$[1]") {
            saw = true;
        }
    }
    assert!(saw);
}

#[test]
fn ensure_ascii_in_nested_arrays() {
    let mut o = opts();
    o.ensure_ascii = true;
    let out = crate::repair_to_string("[['你'], ['好']]", &o).unwrap();
    assert!(!out.chars().any(|c| (c as u32) > 0x7f));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([["你"], ["好"]]));
}

#[test]
fn object_colon_missing_with_unicode_and_comments() {
    let s = "{ '键' /*c*/ '值', x:1 }"; // missing colon for first pair
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["键"], "值");
    assert_eq!(v["x"], 1);
}

#[test]
fn array_trailing_comma_removed() {
    let s = "[1,2,]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2]));
}

#[test]
fn extra_closer_tolerated_at_end() {
    let s = "{a:1}}"; // extra }
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}
