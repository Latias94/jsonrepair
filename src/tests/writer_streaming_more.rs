use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn writer_streaming_object_trailing_comma_fix() {
    let s = "{a:1, b:2, }";
    let o = opts();
    let expect = crate::repair_to_string(s, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(s, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&expect).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&got).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn writer_streaming_array_trailing_comma_fix() {
    let s = "[1,2,3,]";
    let o = opts();
    let expect = crate::repair_to_string(s, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(s, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&expect).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&got).unwrap();
    assert_eq!(v1, v2);
}
