use super::*;

fn opts() -> Options {
    Options::default()
}

fn run_streaming_collect(input: &str, opts: Options) -> String {
    let mut r = crate::StreamRepairer::new(opts);
    let sizes = super::lcg_sizes(98765, input.chars().count());
    let parts = super::chunk_by_char(input, &sizes);
    let mut outs = String::new();
    for p in parts.iter() {
        if let Some(s) = r.push(p).unwrap() {
            outs.push_str(&s);
        }
    }
    if let Some(tail) = r.flush().unwrap() {
        outs.push_str(&tail);
    }
    outs
}

#[test]
fn fuzz_array_unicode_comments_concat() {
    let input = "[ '你'/*x*/+'好', //c\n 'a'+'b', /*m*/ 1, 2 ]";
    let o = opts();
    let ns = crate::repair_to_string(input, &o).unwrap();
    let st = run_streaming_collect(input, o);
    let v1: serde_json::Value = serde_json::from_str(&ns).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn fuzz_object_many_spaces_newlines_and_comments() {
    let input = "{ a: 'x' + 'y' /*c*/ , \n\n b: /re+/ , \r\n c: 1, d: 2 }";
    let o = opts();
    let ns = crate::repair_to_string(input, &o).unwrap();
    let st = run_streaming_collect(input, o);
    let v1: serde_json::Value = serde_json::from_str(&ns).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn fuzz_jsonp_fenced_unicode_mix_random_chunks() {
    let input = "cb(```json\n{ t: '你'+'好', r: /a+/ }\n```);\n";
    let o = opts();
    let st = run_streaming_collect(input, o);
    let v: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v["t"], "你好");
}

#[test]
fn fuzz_ndjson_values_mixed_empty_and_comments() {
    let input = "# h\n{a:1}\n\n// x\n{b:2}\n/*m*/\n{c:3}\n";
    let o = Options {
        stream_ndjson_aggregate: true,
        ..Default::default()
    };
    let st = run_streaming_collect(input, o);
    let v: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v.as_array().map(|a| a.len()), Some(3));
}

#[test]
fn fuzz_large_array_of_pairs_with_comments() {
    let mut raw = String::from("[");
    for i in 0..200usize {
        if i > 0 {
            raw.push_str(",/*c*/");
        }
        raw.push_str(&format!("{{k:{}}}", i));
    }
    raw.push(']');
    let o = opts();
    let ns = crate::repair_to_string(&raw, &o).unwrap();
    let st = run_streaming_collect(&raw, o);
    let v1: serde_json::Value = serde_json::from_str(&ns).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn fuzz_unicode_near_comment_markers_random_chunks() {
    let input = "{ '键'/*注释*/ : '值' , arr: [ '你'/*x*/,'好' ] }";
    let o = opts();
    let ns = crate::repair_to_string(input, &o).unwrap();
    let st = run_streaming_collect(input, o);
    let v1: serde_json::Value = serde_json::from_str(&ns).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn fuzz_numbers_tolerances_and_suspicious_tokens() {
    let input = "{ a:.5, b:1., c:1e, d:10-20, e:1/3, f:1.1.1 }";
    let o = opts();
    let ns = crate::repair_to_string(input, &o).unwrap();
    let st = run_streaming_collect(input, o);
    let v1: serde_json::Value = serde_json::from_str(&ns).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn fuzz_writer_streaming_large_object_equiv() {
    let mut s = String::from("{");
    for i in 0..200usize {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("k{}: 'v' + 'x'", i));
    }
    s.push('}');
    let o = opts();
    let expect = crate::repair_to_string(&s, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(&s, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    assert_eq!(expect, got);
}

#[test]
fn fuzz_streaming_random_chunk_sizes_stability() {
    // Mixed content with comments, unicode, regex, and concatenation
    let input = "{a:[1,/*c*/2, '你'+ '好'], r:/ab+/, note:'x'+'y'}";
    let o = opts();
    let ns = crate::repair_to_string(input, &o).unwrap();
    // Two different random sequences to simulate variance
    let mut r1 = crate::StreamRepairer::new(o.clone());
    let sizes1 = super::lcg_sizes(1, input.chars().count());
    let parts1 = super::chunk_by_char(input, &sizes1);
    let mut outs1 = String::new();
    for p in parts1.iter() {
        if let Some(s) = r1.push(p).unwrap() {
            outs1.push_str(&s);
        }
    }
    if let Some(t1) = r1.flush().unwrap() {
        outs1.push_str(&t1);
    }

    let mut r2 = crate::StreamRepairer::new(o);
    let sizes2 = super::lcg_sizes(2, input.chars().count());
    let parts2 = super::chunk_by_char(input, &sizes2);
    let mut outs2 = String::new();
    for p in parts2.iter() {
        if let Some(s) = r2.push(p).unwrap() {
            outs2.push_str(&s);
        }
    }
    if let Some(t2) = r2.flush().unwrap() {
        outs2.push_str(&t2);
    }

    let v_ns: serde_json::Value = serde_json::from_str(&ns).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&outs1).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&outs2).unwrap();
    assert_eq!(v_ns, v1);
    assert_eq!(v_ns, v2);
}
