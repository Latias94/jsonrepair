use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn quote_hex_like_number() {
    let s = "{n:0xFF}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "0xFF");
}

#[test]
fn weird_exponent_with_dot_parsed_base() {
    let s = "{n:1e1.2}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    // current behavior: parse 1e1 (=10) and ignore trailing .2
    assert_eq!(v["n"], serde_json::json!(10.0));
}

#[test]
fn quote_double_dot_number() {
    let s = "{n:1..0}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "1..0");
}

#[test]
fn quote_range_like_number() {
    let s = "{n:1-2}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["n"], "1-2");
}

#[test]
fn malformed_exponent_signs_parsed_base() {
    let s = "{n:2e-+3}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    // current behavior: tolerance drops malformed exponent entirely, keeps base
    assert_eq!(v["n"], 2);
}

#[test]
fn tolerate_leading_trailing_dot_with_unicode_adjacent() {
    let s = "{'名':.5, '值':1.}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["名"], 0.5);
    assert_eq!(v["值"], 1.0);
}
