#![cfg(feature = "llm-compat")]

use jsonrepair::{Options, options::EngineKind, repair_to_string};

#[test]
fn llm_fenced_multiple_blocks_aggregate() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = "Some text\n```\n{a:1}\n```\nMore text\n```JSON\r\n[2,3]\r\n```\n";
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, "[{\"a\":1},[2,3]]");
}

#[test]
fn llm_fenced_single_block_extract() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = "prefix\n```json\n{x:10}\n```\nsuffix\n";
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, "{\"x\":10}");
}

#[test]
fn llm_jsonp_trim_simple() {
    let mut opts = Options::default();
    opts.engine = EngineKind::LlmCompat;
    let inp = "cb( {a: 1,} );";
    let out = repair_to_string(inp, &opts).unwrap();
    assert_eq!(out, "{\"a\":1}");
}
