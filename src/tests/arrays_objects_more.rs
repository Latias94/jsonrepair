#![allow(
    clippy::needless_borrow,
    clippy::useless_asref,
    clippy::redundant_slicing
)]
use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn obj_missing_colon_string_value() {
    let s = "{ 'a' 'b' }";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":"b"}));
}

#[test]
fn obj_missing_colon_boolean_values_python_keywords() {
    let s = "{ 'a' True, 'b' False }";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":true,"b":false}));
}

#[test]
fn obj_nested_missing_colon_with_comments_and_unicode() {
    let s = "{ o: { 'x' /*c*/ 1, 'y' 2, '名' /*c*/ '值' } }";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["o"], serde_json::json!({"x":1,"y":2,"名":"值"}));
}

#[test]
fn arr_missing_commas_with_adjacent_comments() {
    let s = "[ 'a' /*x*/ 'b' /*y*/ 'c' ]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!(["a", "b", "c"]));
}

#[test]
fn arr_numbers_and_regex_literals() {
    let s = "[ 1, 2, /re+/, 3 ]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        v.as_array().unwrap()[2]
            .as_str()
            .unwrap()
            .starts_with("/re+/")
    );
}

#[test]
fn ndjson_with_crlf_and_comments() {
    let s = "# c\r\n{a:1}\r\n// x\r\n{b:2}\r\n";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([{"a":1},{"b":2}]));
}

#[test]
fn fenced_unknown_language_uppercase_and_crlf() {
    let s = "```JSON\r\n{a:1}\r\n```";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn writer_streaming_ensure_ascii_array() {
    let mut o = opts();
    o.ensure_ascii = true;
    let s = "['你','好','😊']";
    let s_ref: &str = &s;
    let expect = crate::repair_to_string(s_ref, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(s_ref, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    assert_eq!(expect, got);
    assert!(!got.chars().any(|c| (c as u32) > 0x7f));
}

#[test]
fn jsonp_double_wrapper_non_streaming() {
    let s = "cb1(cb2({a:1}));";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn obj_many_members_with_newlines_and_comments() {
    let mut s = String::from("{");
    for i in 0..20 {
        if i > 0 {
            s.push_str(",\n/*x*/\n");
        }
        s.push_str(&format!("'k{}' {}", i, i));
    }
    s.push('}');
    let out = crate::repair_to_string(&s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v.as_object().unwrap().len(), 20);
}
