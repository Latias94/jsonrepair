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
fn test_empty_input() {
    let opts = Options::default();

    // 完全空输入 - 返回空字符串
    let result = crate::repair_to_string("", &opts).unwrap();
    assert_eq!(result, "");

    // 只有空白符 - 返回空字符串
    let result = crate::repair_to_string("   ", &opts).unwrap();
    assert_eq!(result, "");

    // 只有换行符 - 返回空字符串
    let result = crate::repair_to_string("\n\n\n", &opts).unwrap();
    assert_eq!(result, "");

    // 只有制表符 - 返回空字符串
    let result = crate::repair_to_string("\t\t\t", &opts).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_empty_input_with_comments() {
    let opts = Options::default();

    // 只有行注释 - 返回空字符串
    let result = crate::repair_to_string("// comment", &opts).unwrap();
    assert_eq!(result, "");

    // 只有块注释 - 返回空字符串
    let result = crate::repair_to_string("/* comment */", &opts).unwrap();
    assert_eq!(result, "");

    // 多个注释 - 返回空字符串
    let result = crate::repair_to_string("// line1\n/* block */ // line2", &opts).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_empty_input_with_fence() {
    let opts = Options::default();

    // 只有空的 fence - 返回空字符串
    let result = crate::repair_to_string("```json\n```", &opts).unwrap();
    assert_eq!(result, "");

    // fence 中只有空白符 - 返回空字符串
    let result = crate::repair_to_string("```json\n   \n```", &opts).unwrap();
    assert_eq!(result, "");
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
