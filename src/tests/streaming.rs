use super::*;

// Migrated from mod.rs: performance-ish streaming fast path sanity checks
#[test]
fn st_perf_array_spaces_before_comma() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s = r.push("[1         ,2]\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!([1, 2]));
}

#[test]
fn st_perf_array_spaces_before_close() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s = r.push("[1,2         ]\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!([1, 2]));
}

#[test]
fn st_perf_object_spaces_before_comma() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s = r.push("{a:1         ,b:2}\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!({"a":1,"b":2}));
}

#[test]
fn st_perf_object_spaces_before_close() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s = r.push("{a:1,b:2         }\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!({"a":1,"b":2}));
}

#[test]
fn st_perf_array_multi_spaces_between_elements() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s = r.push("[1          2          3]\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!([1, 2, 3]));
}

#[test]
fn st_concat_three_with_comments_between() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["\"a\"", "/*c*/ + ", "\"b\" + ", "//x\n", "\"c\"\n"]; // one string
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0], "\"abc\"");
}

#[test]
fn st_regex_split_variant() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["/a", "b+/", "\n"]; // string value
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].trim_end(), "\"/ab+/\"");
}

#[test]
fn st_fence_language_with_trailing_spaces() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["```json   \n", "{a:1}", "\n```\n"]; // one object
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    assert_eq!(outs.len(), 1);
}

#[test]
fn st_trailing_jsonp_artifacts_after_objects() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["{a:1}\n)", ";\n"]; // ignore ) and ; at root
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
}

#[test]
fn st_comments_crlf_then_object() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s1 = r.push("// x\r\n").unwrap();
    assert_eq!(s1, "");
    let s2 = r.push("{a:1}\r\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s2).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_nested_arrays_split() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["[", "[1,2]", ",3]", "\n"]; // one array
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    assert_eq!(outs.len(), 1);
}

#[test]
fn st_root_blank_lines_then_object() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s = r.push("\n\n   \n{a:1}\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_comment_only_then_value_line() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let s1 = r.push("// hi\n").unwrap();
    assert_eq!(s1, "");
    let s2 = r.push("{a:1}\n").unwrap();
    let v: serde_json::Value = serde_json::from_str(&s2).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_ndjson_blank_and_comments_mixture_more() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = [
        "\n", "# a\n", "{x:1}\n", "// b\n\n", "{y:2}\n", "/*c*/\n", "{z:3}\n",
    ]; // 3 objects
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 3);
}

#[test]
fn st_unicode_string_concat_across_chunks_more() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["\"你\"", "+", "\"好\"\n"]; // one string value
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!("你好"));
}

#[test]
fn st_regex_literal_split_with_flags() {
    let mut r = crate::StreamRepairer::new(Options::default());
    // Embed regex into object for robust parsing
    let parts = ["{r:", "/a", "b+", "/i}", "\n"]; // emits one object
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    let rstr = v.get("r").and_then(|x| x.as_str()).unwrap();
    assert!(rstr == "/ab+/i" || rstr == "/ab+/");
}

#[test]
fn st_writer_basic_and_aggregate() {
    // NDJSON without aggregate: writer receives multiple JSON texts
    let mut corpus = String::new();
    for i in 0..20usize {
        corpus.push_str(&format!("{{a:{}}}\n", i));
    }
    let mut r1 = crate::StreamRepairer::new(Options::default());
    let sizes = super::lcg_sizes(7777, corpus.len());
    let parts = super::chunk_by_char(&corpus, &sizes);
    let mut buf = Vec::new();
    for p in parts.iter() {
        r1.push_to_writer(p, &mut buf).unwrap();
    }
    r1.flush_to_writer(&mut buf).unwrap();
    // Parse concatenated JSON texts
    let s = String::from_utf8(buf).unwrap();
    let de = serde_json::Deserializer::from_str(&s).into_iter::<serde_json::Value>();
    let mut cnt = 0;
    for res in de {
        res.unwrap();
        cnt += 1;
    }
    assert!(cnt >= 20);

    // Aggregate mode: should produce a single JSON array on flush
    let mut opts = Options::default();
    let mut __tmp = opts;
    __tmp.stream_ndjson_aggregate = true;
    opts = __tmp;
    let mut r2 = crate::StreamRepairer::new(opts);
    let sizes2 = super::lcg_sizes(8888, corpus.len());
    let parts2 = super::chunk_by_char(&corpus, &sizes2);
    let mut buf2 = Vec::new();
    for p in parts2.iter() {
        r2.push_to_writer(p, &mut buf2).unwrap();
    }
    r2.flush_to_writer(&mut buf2).unwrap();
    let s2 = String::from_utf8(buf2).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s2).unwrap();
    assert_eq!(v.as_array().map(|a| a.len()), Some(20));
}

#[test]
fn st_bom_ignored_before_value() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let outs = [
        r.push("\u{FEFF}").unwrap(),
        r.push("{a:1}\n").unwrap(),
        r.flush().unwrap(),
    ];
    let merged: String = outs.concat();
    let v: serde_json::Value = serde_json::from_str(&merged).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_chunks_pure_fn_basic() {
    let parts = ["callback(", "{a:1}", ");\n", "{b:2}\n", "#c\n", "{c:3}\n"];
    let mut opts = Options::default();
    // Default (non-aggregate): treat the later objects as NDJSON values
    let s = crate::repair_chunks_to_string(parts.as_slice().iter().copied(), &opts).unwrap();
    // Parse multiple concatenated JSON texts
    let mut de = serde_json::Deserializer::from_str(&s).into_iter::<serde_json::Value>();
    let v1 = de.next().unwrap().unwrap(); // from callback({a:1})
    let v2 = de.next().unwrap().unwrap(); // {b:2}
    let v3 = de.next().unwrap().unwrap(); // {c:3}
    assert_eq!(v1, serde_json::json!({"a":1}));
    assert_eq!(v2, serde_json::json!({"b":2}));
    assert_eq!(v3, serde_json::json!({"c":3}));

    // Aggregate mode: NDJSON values are collected into a single array
    let mut __tmp = opts;
    __tmp.stream_ndjson_aggregate = true;
    opts = __tmp;
    let s2 = crate::repair_chunks_to_string(parts.as_slice().iter().copied(), &opts).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s2).unwrap();
    let arr = v.as_array().expect("array");
    // In aggregate mode, all root-level values (including JSONP's inner value) are aggregated
    assert_eq!(arr.len(), 3);
}
