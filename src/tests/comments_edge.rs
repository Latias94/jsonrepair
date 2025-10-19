use super::*;

fn opts() -> Options {
    Options::default()
}

#[test]
fn comments_only_lines_before_and_after() {
    let s = "// head\n/* mid */\n{a:1}\n# tail\n";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn comments_inside_array_near_closer() {
    let s = "[1,2/* end */]";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([1, 2]));
}

#[test]
fn hash_comment_between_members() {
    let s = "{a:1\n# x\n,b:2}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1,"b":2}));
}

#[test]
fn unicode_in_comments_adjacent_to_colon() {
    let s = "{'键'/*注释*/:/*注释*/'值'}";
    let out = crate::repair_to_string(s, &opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"键":"值"}));
}
