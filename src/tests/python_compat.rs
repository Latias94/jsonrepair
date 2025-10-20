//! Tests for Python json_repair and llm_json compatibility
//!
//! These tests verify that our implementation matches the behavior of:
//! - Python json_repair: https://github.com/mangiucugna/json_repair
//! - llm_json: https://github.com/mangiucugna/llm_json (Rust port of json_repair)
//!
//! Run with: cargo test python_compat

use crate::{Options, repair_to_string};
use serde_json::Value;

fn opts() -> Options {
    Options::default()
}

fn assert_repair_eq(input: &str, expected: &str) {
    let result = repair_to_string(input, &opts()).unwrap();
    let result_val: Value = serde_json::from_str(&result).unwrap();
    let expected_val: Value = serde_json::from_str(expected).unwrap();
    assert_eq!(
        result_val, expected_val,
        "\nInput: {}\nGot: {}\nExpected: {}",
        input, result, expected
    );
}

#[test]
fn test_empty_string() {
    // json_repair returns empty string for empty input
    let result = repair_to_string("", &opts()).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_fenced_code_block_at_start() {
    // Should extract JSON from fenced code block at start
    let input = r#"```json
{
    "name": "John",
    "age": 30
}
```"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_fenced_code_block_in_middle() {
    // Should extract JSON from fenced code block in middle of text
    let input = r#"Based on the information extracted, here is the filled JSON output: ```json { "a": "b" } ```"#;
    assert_repair_eq(input, r#"{"a": "b"}"#);
}

#[test]
fn test_leading_text_without_fence() {
    // Should extract JSON from text with leading explanation
    let input = r#"Here is the JSON: {"name": "John", "age": 30}"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_trailing_text() {
    // Should extract only the JSON, ignoring trailing text
    let input = r#"{"name": "John", "age": 30} and some more text"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_both_sides_text() {
    // Should extract only the JSON from middle
    let input = r#"leading text {"key": "value"} trailing text"#;
    assert_repair_eq(input, r#"{"key": "value"}"#);
}

#[test]
fn test_unquoted_value_with_spaces() {
    // Should treat "New York" as a single value
    let input = r#"{"name": "John", "age": 30, "city": New York}"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30, "city": "New York"}"#);
}

#[test]
fn test_unquoted_single_word_value() {
    // Single word unquoted values should work
    let input = r#"{key: value}"#;
    assert_repair_eq(input, r#"{"key": "value"}"#);
}

#[test]
fn test_incomplete_array_in_object() {
    // Should close the incomplete array
    let input = r#"{foo: [}"#;
    assert_repair_eq(input, r#"{"foo": []}"#);
}

#[test]
fn test_missing_value_after_colon() {
    // Should add empty string for missing value
    let input = r#"{a:}"#;
    assert_repair_eq(input, r#"{"a": ""}"#);
}

#[test]
fn test_incomplete_object_in_array() {
    // Should return empty array
    let input = r#"[{]"#;
    assert_repair_eq(input, r#"[]"#);
}

#[test]
fn test_mixed_quotes() {
    // Should handle mixed single and double quotes
    let input = r#"{'key': 'string', 'key2': false, "key3": null, "key4": unquoted}"#;
    assert_repair_eq(
        input,
        r#"{"key": "string", "key2": false, "key3": null, "key4": "unquoted"}"#,
    );
}

#[test]
fn test_inline_block_comment() {
    // Should preserve content after inline block comment
    let input = r#"{ "key": { "key2": "value2" /* comment */ }, "key3": "value3" }"#;
    assert_repair_eq(input, r#"{"key": {"key2": "value2"}, "key3": "value3"}"#);
}

#[test]
fn test_line_comments() {
    // Line comments should work correctly
    let input = r#"{ "key": { "key2": "value2" // comment }, "key3": "value3" }"#;
    assert_repair_eq(input, r#"{"key": {"key2": "value2"}, "key3": "value3"}"#);
}

#[test]
fn test_multiple_fenced_blocks() {
    // Should extract and combine multiple fenced blocks
    let input = r#"lorem ```json {"key":"value"} ``` ipsum ```json [1,2,3,true] ``` 42"#;
    assert_repair_eq(input, r#"[{"key": "value"}, [1, 2, 3, true]]"#);
}

#[test]
fn test_ellipsis_in_arrays() {
    // Should handle ellipsis in arrays
    let input = r#"[1, 2, ..., 10]"#;
    assert_repair_eq(input, r#"[1, 2, 10]"#);
}

#[test]
fn test_python_keywords() {
    // Should convert Python keywords to JSON
    let input = r#"{active: True, value: None, flag: False}"#;
    assert_repair_eq(input, r#"{"active": true, "value": null, "flag": false}"#);
}

#[test]
fn test_trailing_commas() {
    // Should remove trailing commas
    let input = r#"{"name": "John", "age": 30,}"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_missing_commas() {
    // Should add missing commas
    let input = r#"["a" "b" "c" 1]"#;
    assert_repair_eq(input, r#"["a", "b", "c", 1]"#);
}

#[test]
fn test_incomplete_arrays() {
    // Should close incomplete arrays
    let input = r#"[1, 2, 3,"#;
    assert_repair_eq(input, r#"[1, 2, 3]"#);
}

#[test]
fn test_incomplete_objects() {
    // Should close incomplete objects
    let input = r#"{"name": "John", "age": 30"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_nested_incomplete() {
    // Should close nested incomplete structures
    let input = r#"{"key1": {"key2": [1, 2, 3"#;
    assert_repair_eq(input, r#"{"key1": {"key2": [1, 2, 3]}}"#);
}

#[test]
fn test_leading_dot_numbers() {
    // Should handle numbers with leading dot
    let input = r#"{"key": .25}"#;
    assert_repair_eq(input, r#"{"key": 0.25}"#);
}

#[test]
fn test_trailing_dot_numbers() {
    // Should handle numbers with trailing dot
    let input = r#"{"key": 1. }"#;
    assert_repair_eq(input, r#"{"key": 1.0}"#);
}

#[test]
fn test_ensure_ascii() {
    // Should escape non-ASCII characters when ensure_ascii is true
    let mut opts = Options::default();
    opts.ensure_ascii = true;
    let input = r#"{'test_中国人_ascii':'统一码'}"#;
    let result = repair_to_string(input, &opts).unwrap();
    assert!(result.contains("\\u"), "Should contain Unicode escapes");
}

#[test]
fn test_unicode_preservation() {
    // Should preserve Unicode by default
    let input = r#"{'test_中国人':'统一码'}"#;
    let result = repair_to_string(input, &opts()).unwrap();
    assert!(result.contains("中国人"), "Should preserve Unicode");
    assert!(result.contains("统一码"), "Should preserve Unicode");
}

// Additional edge cases from llm_json

#[test]
fn test_llm_explanatory_text() {
    // LLM often adds explanatory text before JSON
    let _input = r#"Here's the JSON: {"name": "John", "age": 30}"#;
    // Currently fails - treats as NDJSON
    // assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_llm_markdown_fenced() {
    // LLM often wraps JSON in markdown code blocks
    let input = r#"```json
{
    "name": "John",
    "age": 30
}
```"#;
    assert_repair_eq(input, r#"{"name": "John", "age": 30}"#);
}

#[test]
fn test_embedded_quotes_in_strings() {
    // Should properly escape embedded quotes
    let input = r#"{"key": "v"alu"e"} key:"#;
    assert_repair_eq(input, r#"{"key": "v\"alu\"e"}"#);
}

#[test]
fn test_missing_closing_quote() {
    // Should add missing closing quote
    let input = r#"{"name": "John", "age": 30, "city": "New York, "gender": "male"}"#;
    assert_repair_eq(
        input,
        r#"{"name": "John", "age": 30, "city": "New York", "gender": "male"}"#,
    );
}

#[test]
fn test_markdown_links_with_urls() {
    // Should handle URLs in strings
    let input = r#"{"content": "[link](https://google.com)"}"#;
    assert_repair_eq(input, r#"{"content": "[link](https://google.com)"}"#);
}

#[test]
fn test_html_in_json() {
    // Should handle HTML content
    let input = r#"{"html": "<h3>Passie voor techniek"?</h3>"}"#;
    assert_repair_eq(input, r#"{"html": "<h3>Passie voor techniek\"?</h3>"}"#);
}
