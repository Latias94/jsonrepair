#![cfg(feature = "llm-compat")]

use jsonrepair::{Options, options::EngineKind, repair_to_string};

#[test]
fn llm_python_keywords_allowed() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let out = repair_to_string("[True, False, None]", &opts).unwrap();
    assert_eq!(out, "[true,false,null]");
}

#[test]
fn llm_python_keywords_disallowed() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    opts.allow_python_keywords = false;
    let out = repair_to_string("[True, False, None]", &opts).unwrap();
    assert_eq!(out, "[\"True\",\"False\",\"None\"]");
}

#[test]
fn llm_js_nonfinite_normalize() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let out = repair_to_string("[NaN, Infinity, -Infinity]", &opts).unwrap();
    assert_eq!(out, "[null,null,null]");
}

#[test]
fn llm_js_nonfinite_disabled() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    opts.normalize_js_nonfinite = false;
    let out = repair_to_string("[NaN, Infinity, -Infinity]", &opts).unwrap();
    // NaN/Infinity 将被视为符号转字符串；-Infinity 走数值路径后回退为字符串
    assert_eq!(out, "[\"NaN\",\"Infinity\",\"-Infinity\"]");
}

#[test]
fn llm_undefined_to_null_toggle() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let out = repair_to_string("[undefined]", &opts).unwrap();
    assert_eq!(out, "[null]");
    opts.repair_undefined = false;
    let out2 = repair_to_string("[undefined]", &opts).unwrap();
    assert_eq!(out2, "[\"undefined\"]");
}
