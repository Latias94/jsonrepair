use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn ensure_ascii_mixed_cjk_emoji() {
    let mut o = opts();
    o.ensure_ascii = true;
    let out = crate::repair_to_string("{s:'你好😊'}", &o).unwrap();
    assert!(!out.chars().any(|c| (c as u32) > 0x7f));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["s"], "你好😊");
}

#[test]
fn unicode_escape_lower_upper() {
    let s = "{a:'\\u4f60\\u597d', b:'\\u4F60\\u597D'}"; // 你好
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["a"], "你好");
    assert_eq!(v["b"], "你好");
}

#[test]
fn surrogate_pair_emoji() {
    let s = "{e:'\\uD83D\\uDE00'}"; // 😀
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["e"], "😀");
}

#[test]
fn mixed_backslashes_and_quotes() {
    let s = r"{p:'C:\\Program Files\\Apps\\bin'}"; // backslashes and spaces
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        v["p"]
            .as_str()
            .unwrap()
            .contains("C:\\Program Files\\Apps\\bin")
    );
}

#[test]
fn control_chars_escaped() {
    let s = "{t:'a\\tb\\nc\\rd\\fe'}"; // \t \n \r \f
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["t"], "a\tb\nc\rd\u{000C}e");
}

#[test]
fn string_concat_with_escapes() {
    let s = "'a\\n' + 'b\\t'"; // escaped newline + tab
    let out = crate::repair_to_string(s, &opts()).unwrap();
    assert_eq!(out, "\"a\\nb\\t\"");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!("a\nb\t"));
}

#[test]
fn ensure_ascii_on_surrogate_pair() {
    let mut o = opts();
    o.ensure_ascii = true;
    let out = crate::repair_to_string("{e:'😀'}", &o).unwrap();
    assert!(!out.chars().any(|c| (c as u32) > 0x7f));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["e"], "😀");
}
