use super::*;

#[test]
fn ns_bom_is_ignored_at_start() {
    let s = "\u{FEFF}{a:1}\n".to_string();
    let out = crate::repair_to_string(&s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn ns_js_nonfinite_to_null() {
    let s = "{x:NaN, y:Infinity, z:-Infinity}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"x":null, "y":null, "z":null}));
}

#[test]
fn ns_writer_roundtrip() {
    let s = "{'a': 1, b: 'x', /*c*/ arr: [1,2,3]}";
    let mut buf = Vec::new();
    crate::repair_to_writer(s, &Options::default(), &mut buf).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1, "b":"x", "arr":[1,2,3]}));
}
