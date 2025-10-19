use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cargo_bin() -> &'static str {
    // The main binary name matches the package: jsonrepair
    "jsonrepair"
}

#[test]
fn cli_stdin_stdout_basic() {
    let mut cmd = Command::cargo_bin(cargo_bin()).unwrap();
    let input = "{'a':1, b: 'x'}\n";
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|out: &[u8]| {
            std::str::from_utf8(out)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .is_some()
        }));
}

#[test]
fn cli_file_to_file_stream() {
    let dir = tempdir().unwrap();
    let inp = dir.path().join("in.json");
    let out = dir.path().join("out.json");
    fs::write(&inp, "{a:1}\n{b:2}\n").unwrap();
    Command::cargo_bin(cargo_bin())
        .unwrap()
        .args([
            "--stream",
            "--chunk-size",
            "16",
            inp.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();
    let s = fs::read_to_string(out).unwrap();
    // concatenated JSON texts: parse with serde stream
    let mut de = serde_json::Deserializer::from_str(&s).into_iter::<serde_json::Value>();
    let v1 = de.next().unwrap().unwrap();
    let v2 = de.next().unwrap().unwrap();
    assert_eq!(v1, serde_json::json!({"a":1}));
    assert_eq!(v2, serde_json::json!({"b":2}));
}

#[test]
fn cli_in_place_and_pretty() {
    let dir = tempdir().unwrap();
    let inp = dir.path().join("inplace.json");
    fs::write(&inp, "{'a':1, b:2}").unwrap();
    // in-place non-pretty
    Command::cargo_bin(cargo_bin())
        .unwrap()
        .args(["--in-place", inp.to_str().unwrap()])
        .assert()
        .success();
    let s = fs::read_to_string(&inp).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!({"a":1,"b":2}));
    // pretty print
    Command::cargo_bin(cargo_bin())
        .unwrap()
        .args(["--in-place", "--pretty", inp.to_str().unwrap()])
        .assert()
        .success();
    let s2 = fs::read_to_string(&inp).unwrap();
    assert!(s2.contains("\n") && s2.contains("  "));
}

#[test]
fn cli_stream_stdin_concat() {
    let mut cmd = Command::cargo_bin(cargo_bin()).unwrap();
    let input = "{a:1}\n{b:2}\n";
    cmd.args(["--stream"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::function(|out: &[u8]| {
            if let Ok(s) = std::str::from_utf8(out) {
                let mut de = serde_json::Deserializer::from_str(s).into_iter::<serde_json::Value>();
                return de.next().is_some() && de.next().is_some();
            }
            false
        }));
}

#[test]
fn cli_stream_ndjson_aggregate_file_stdout() {
    let dir = tempdir().unwrap();
    let inp = dir.path().join("agg.jsonl");
    fs::write(&inp, "{a:1}\n{b:2}\n").unwrap();
    let assert = Command::cargo_bin(cargo_bin())
        .unwrap()
        .args(["--stream", "--ndjson-aggregate", inp.to_str().unwrap()])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v.as_array().map(|a| a.len()), Some(2));
}

#[test]
fn cli_word_comment_multiple_markers() {
    let dir = tempdir().unwrap();
    let inp = dir.path().join("markers.json");
    fs::write(&inp, "{ 'a':1, COMMENT 'b':2, SHOULD_NOT_EXIST 'c':3 }").unwrap();
    let assert = Command::cargo_bin(cargo_bin())
        .unwrap()
        .args([
            "--word-comment",
            "COMMENT",
            "--word-comment",
            "SHOULD_NOT_EXIST",
            inp.to_str().unwrap(),
        ])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1, "b":2, "c":3}));
}
