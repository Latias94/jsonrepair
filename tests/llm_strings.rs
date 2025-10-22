#![cfg(feature = "llm-compat")]

use jsonrepair::{Options, options::EngineKind, repair_to_string};

#[test]
fn llm_concat_basic_and_comments() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = r#""a" + /*c*/ 'b'"#;
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, "\"ab\"");
}

#[test]
fn llm_concat_newline_plus() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = "'a'\n+ 'b'";
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, "\"ab\"");
}

#[test]
fn llm_concat_escapes_preserved() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = "'a\\n' + 'b\\t'";
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, "\"a\\nb\\t\"");
}

#[test]
fn llm_regex_literal_to_string() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = "{r:/ab+/gi}";
    let out = repair_to_string(inp, &opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let r = v.get("r").and_then(|x| x.as_str()).unwrap();
    assert!(r == "/ab+/gi" || r == "/ab+/" || r.contains("/ab+/"));
}
