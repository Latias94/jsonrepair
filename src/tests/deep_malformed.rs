use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn deep_missing_colon_and_commas_with_unicode_and_comments() {
    let s = "{ a: [ { '名' /*c*/ '值' } /*m*/ , { 'x' 1 'y' 2 } ] }";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"][0], serde_json::json!({"名":"值"}));
    assert_eq!(v["a"][1], serde_json::json!({"x":1,"y":2}));
}

#[test]
fn deep_extra_closers_and_trailing_commas() {
    let s = "{ a:[1,2,], b:{x:1,}, }}}"; // multiple extra closers at end
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], serde_json::json!([1, 2]));
    if let Some(obj) = v.get("b").and_then(|x| x.as_object()) {
        assert_eq!(obj.get("x"), Some(&serde_json::json!(1)));
    }
}

#[test]
fn deep_ndjson_with_fenced_and_jsonp_streaming_aggregate() {
    let mut o = opts();
    o.stream_ndjson_aggregate = true;
    let mut r = crate::StreamRepairer::new(o);
    let parts = ["```json\n", "{a:1}", "\n```\n", "cb({b:2});\n", "{c:3}\n"];
    for p in parts.iter() {
        let _ = r.push(p).unwrap();
    }
    let out = r.flush().unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([{"a":1},{"b":2},{"c":3}]));
}

#[test]
fn deep_aggressive_truncation_fix_nested() {
    let mut o = opts();
    o.aggressive_truncation_fix = true;
    let s = "{root:{a:[{b:1}, {c:2}"; // truncated
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v.get("root").is_some());
    // at least retain first object in array
    assert_eq!(v["root"]["a"][0], serde_json::json!({"b":1}));
}

#[test]
fn deep_whitespace_runs_and_fastpaths_no_regression() {
    // long runs of spaces/tabs/newlines between tokens in nested containers
    let mut s = String::from("{");
    s.push_str("k: [");
    for i in 0..50usize {
        if i > 0 {
            s.push_str("\n\n   \t \n");
        }
        s.push_str(&format!("{{i:{}}}", i));
    }
    s.push_str("]}");
    let out = crate::repair_to_string(&s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["k"].as_array().unwrap().len(), 50);
}

#[test]
fn deep_string_concat_with_unicode_and_comments() {
    let s = "{name:'你'/*c*/+'好', arr:['a'+ 'b', 'x' + 'y']}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["name"], "你好");
    assert_eq!(v["arr"], serde_json::json!(["ab", "xy"]));
}
