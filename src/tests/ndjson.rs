use super::*;

#[test]
fn st_multiple_values_with_blank_and_comments() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["{a:1}\n", "# blank\n\n", "{b:2}\n", "// c\n", "{c:3}\n"];
    let mut outs = Vec::new();
    for p in parts.iter() { if let Some(s) = r.push(p).unwrap() { outs.push(s); } }
    if let Some(tail) = r.flush().unwrap() { outs.push(tail); }
    assert_eq!(outs.len(), 3);
}

#[test]
fn st_ndjson_objects_and_arrays_mixed() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["{a:1}\n", "[1,2]\n", "{b:2}\n"];
    let mut outs = Vec::new();
    for p in parts.iter() { if let Some(s) = r.push(p).unwrap() { outs.push(s); } }
    if let Some(tail) = r.flush().unwrap() { outs.push(tail); }
    assert_eq!(outs.len(), 3);
}

#[test]
fn st_ndjson_with_comments_and_blanks() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["{a:1}\n", "# x\n\n", "{b:2}\n", "// y\n", "{c:3}\n"]; // three objects
    let mut outs = Vec::new();
    for p in parts.iter() { if let Some(s) = r.push(p).unwrap() { outs.push(s); } }
    if let Some(tail) = r.flush().unwrap() { outs.push(tail); }
    assert_eq!(outs.len(), 3);
}

#[test]
fn st_ndjson_aggregate_mode_produces_single_array() {
    let mut corpus = String::new();
    for i in 0..30usize {
        corpus.push_str(&format!("{{a:{}}}\n", i));
    }
    let mut opts = Options::default();
    let mut __tmp = opts;
    __tmp.stream_ndjson_aggregate = true;
    opts = __tmp;
    let mut r = crate::StreamRepairer::new(opts);
    let sizes = super::lcg_sizes(24601, corpus.len());
    let parts = super::chunk_by_char(&corpus, &sizes);
    let mut outs = Vec::new();
    for p in parts.iter() { if let Some(s) = r.push(p).unwrap() { outs.push(s); } }
    assert!(outs.is_empty());
    let ret = r.flush().unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&ret).unwrap();
    let arr = v.as_array().expect("aggregate returns array");
    assert_eq!(arr.len(), 30);
}

#[test]
fn st_ndjson_aggregate_numbers_and_arrays() {
    let mut corpus = String::new();
    corpus.push_str("1\n");
    corpus.push_str("[2,3]\n");
    corpus.push_str("{x:4}\n");
    let mut opts = Options::default();
    let mut __tmp = opts;
    __tmp.stream_ndjson_aggregate = true;
    opts = __tmp;
    let mut r = crate::StreamRepairer::new(opts);
    let sizes = super::lcg_sizes(13579, corpus.len());
    let parts = super::chunk_by_char(&corpus, &sizes);
    for p in parts.iter() {
        let _ = r.push(p).unwrap();
    }
    let ret = r.flush().unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&ret).unwrap();
    let arr = v.as_array().expect("array");
    assert!(arr.len() >= 2);
}

#[test]
fn st_ndjson_numbers_only_many_small_chunks() {
    let mut corpus = String::new();
    for i in 0..100usize {
        corpus.push_str(&format!("{}\n", i));
    }
    let opts = Options {
        stream_ndjson_aggregate: true,
        ..Default::default()
    };
    let mut r = crate::StreamRepairer::new(opts);
    let sizes = super::lcg_sizes(3, corpus.len());
    let parts = super::chunk_by_char(&corpus, &sizes);
    for p in parts.iter() {
        let _ = r.push(p).unwrap();
    }
    let ret = r.flush().unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&ret).unwrap();
    let arr = v.as_array().expect("array");
    assert!(!arr.is_empty());
}
