use super::*;

#[test]
fn ns_numbers_scientific_mix() {
    let s = "[1e+2, -3.5e-1, 6E0]";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([100.0, -0.35, 6.0]));
}

#[test]
fn ns_tolerance_leading_dot_and_trailing_dot_and_incomplete_exp() {
    // .25 -> 0.25
    let out1 = crate::repair_to_string("{a:.25}", &Options::default()).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&out1).unwrap();
    assert_eq!(v1, serde_json::json!({"a":0.25}));
    // 1. -> 1.0
    let out2 = crate::repair_to_string("{a:1.}", &Options::default()).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&out2).unwrap();
    assert_eq!(v2, serde_json::json!({"a":1.0}));
    // 1e -> 1 (drop exponent)
    let out3 = crate::repair_to_string("{a:1e}", &Options::default()).unwrap();
    let v3: serde_json::Value = serde_json::from_str(&out3).unwrap();
    assert_eq!(v3, serde_json::json!({"a":1}));
}

#[test]
fn ns_suspicious_numbers_quoted_as_string() {
    let cases = vec![
        ("{x:1/3}", serde_json::json!({"x":"1/3"})),
        ("{x:10-20}", serde_json::json!({"x":"10-20"})),
        ("{x:1.1.1}", serde_json::json!({"x":"1.1.1"})),
        ("{x:2notanumber}", serde_json::json!({"x":"2notanumber"})),
    ];
    for (inp, want) in cases {
        let out = crate::repair_to_string(inp, &Options::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v, want, "input={}", inp);
    }
}

#[test]
fn ns_numbers_scientific_sign_variants() {
    let s = "[1e2, -2e-1, 6E+1]";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([100.0, -0.2, 60.0]));
}

#[test]
fn ns_numbers_in_objects_table() {
    let s = "{a:1.5, b:-2.25e1, c:6E0}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"a":1.5, "b":-22.5, "c":6.0}));
}

#[test]
fn ns_numbers_mixed_signs_and_decimals_array() {
    let s = "[0.1, -2.0, 3.15, -0.25e2]";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([0.1, -2.0, 3.15, -25.0]));
}

#[test]
fn ns_numbers_upper_e_plus_sign() {
    let s = "[1E+2, 2E-1]";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([100.0, 0.2]));
}

#[test]
fn ns_numbers_nested_arrays_with_quote_policy() {
    let o = Options {
        leading_zero_policy: LeadingZeroPolicy::QuoteAsString,
        ..Default::default()
    };
    let out = crate::repair_to_string("[[007],[08,0]]", &o).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!([["007"], ["08", 0]]));
}

#[test]
fn ns_numbers_leading_zero_quote_policy_in_array() {
    let mut opts = Options::default();
    let mut __tmp = opts;
    __tmp.leading_zero_policy = LeadingZeroPolicy::QuoteAsString;
    opts = __tmp;
    let out = crate::repair_to_string("[01, 007, 0]", &opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!(["01", "007", 0]));
}

#[test]
fn ns_numbers_leading_zero_keep_policy_preserved() {
    let out = crate::repair_to_string("[007,08]", &Options::default()).unwrap();
    // default policy keeps numbers with leading zeros as numbers (pragmatic)
    assert_eq!(out, "[007,08]");
}

#[test]
fn ns_unquoted_key_starting_with_digit() {
    let s = "{0a:1, 9b:2}";
    let out = crate::repair_to_string(s, &Options::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v, serde_json::json!({"0a":1, "9b":2}));
}
