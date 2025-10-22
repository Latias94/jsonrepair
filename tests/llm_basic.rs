#![cfg(feature = "llm-compat")]

use jsonrepair::{Options, options::EngineKind, repair_to_string};

#[test]
fn llm_object_missing_colon_and_comma() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = r#"{a 1 b:2 c 3}"#;
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, r#"{"a":1,"b":2,"c":3}"#);
}

#[test]
fn llm_object_redundant_commas_and_trailing() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = r#"{a:1,,b:2,}"#;
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, r#"{"a":1,"b":2}"#);
}

#[test]
fn llm_array_missing_commas_and_redundant() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = r#"[1 2,, 3,]"#;
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, r#"[1,2,3]"#);
}
