use super::*;

#[test]
fn ns_comments_around_delimiters() {
    // Comments around colon/commas but not inside key token
    let s = "{a:/*y*/1/*z*/,/*w*/b:2}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1,"b":2}));
}

#[test]
fn ns_comments_between_brackets() {
    let s = "{/*c*/'x'/*c*/:/*c*/'y'/*c*/}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"x":"y"}));
}

#[test]
fn ns_hash_comment_line_inside_array() {
    let s = "[1\n# skip\n,2]";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2]));
}

#[test]
fn ns_unicode_adjacent_to_colon_and_comma() {
    let s = "{\"你好\":1, b:2, c:3}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"你好":1,"b":2,"c":3}));
}

#[test]
fn ns_comments_between_values_and_commas() {
    let s = "[1/*x*/,/*y*/2/*z*/,3]";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2, 3]));
}

#[test]
fn ns_hash_comments_mixed_crlf_and_blank_lines() {
    let s = "#h\r\n\r\n{a:1}\r\n#t\r\n{b:2}\n";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    // Non-streaming wraps multiple root values into an array
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], serde_json::json!({"a":1}));
    assert_eq!(arr[1], serde_json::json!({"b":2}));
}

#[test]
fn ns_multiple_comments_around_colon_comma_variants() {
    let s = "{ 'a' /*x*/ : /*y*/ 1 /*z*/ , /*w*/ 'b' : 2 }";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1, "b":2}));
}

#[test]
fn ns_hash_comments_inside_object_lines_single_value() {
    let s = "#start\n{ a:1 }\n#end\n";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn ns_word_comment_markers_skip_before_key() {
    let s = "{\"value_1\": true, COMMENT \"value_2\": \"data\"}";
    let o = Options {
        word_comment_markers: vec!["COMMENT".to_string(), "SHOULD_NOT_EXIST".to_string()],
        ..Default::default()
    };
    let out = crate::repair_to_string(s, &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"value_1": true, "value_2": "data"}));
}
