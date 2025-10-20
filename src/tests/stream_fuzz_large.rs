use super::*;

fn opts() -> Options {
    Options::default()
}

fn gen_large_corpus(repeat: usize) -> String {
    let unit = "{obj:{a:1,b:'你'+'好',r:/ab+/,arr:[/*c*/1,2,3]}, x:2}\n";
    let mut s = String::with_capacity(unit.len() * repeat);
    for _ in 0..repeat {
        s.push_str(unit);
    }
    s
}

#[test]
fn large_streaming_vs_non_streaming_equivalence() {
    // Keep size moderate for CI (~2-3 MiB)
    let raw = gen_large_corpus(40000 / 10); // tune to ~2MB depending on unit size
    let o = opts();
    let ns = crate::repair_to_string(&raw, &o).unwrap();
    let mut r = crate::StreamRepairer::new(o);
    let sizes = super::lcg_sizes(314159, raw.chars().count());
    let parts = super::chunk_by_char(&raw, &sizes);
    let mut st = String::new();
    for p in parts.iter() {
        if let Some(s) = r.push(p).unwrap() {
            st.push_str(&s);
        }
    }
    if let Some(tail) = r.flush().unwrap() {
        st.push_str(&tail);
    }
    // Parse concatenated JSON texts into vectors for fair comparison
    fn parse_concat(s: &str) -> Vec<serde_json::Value> {
        let de = serde_json::Deserializer::from_str(s).into_iter::<serde_json::Value>();
        let mut out = Vec::new();
        for v in de {
            let v = v.unwrap();
            if let Some(arr) = v.as_array() {
                out.extend_from_slice(arr);
            } else {
                out.push(v);
            }
        }
        out
    }
    let v1 = parse_concat(&ns);
    let v2 = parse_concat(&st);
    assert_eq!(v1, v2);
}

#[test]
fn large_ndjson_aggregate_streaming_vs_non_streaming() {
    let mut raw = String::new();
    for i in 0..20000 {
        raw.push_str(&format!("{{k:{}}}\n", i));
    } // ~1MB
    let o = Options {
        stream_ndjson_aggregate: true,
        ..Default::default()
    };
    let mut r = crate::StreamRepairer::new(o);
    let sizes = super::lcg_sizes(271828, raw.chars().count());
    let parts = super::chunk_by_char(&raw, &sizes);
    let mut st = String::new();
    for p in parts.iter() {
        if let Some(s) = r.push(p).unwrap() {
            st.push_str(&s);
        }
    }
    if let Some(tail) = r.flush().unwrap() {
        st.push_str(&tail);
    }
    let v: serde_json::Value = serde_json::from_str(&st).unwrap();
    assert_eq!(v.as_array().map(|a| a.len()), Some(20000));
}

#[test]
fn large_writer_streaming_array_equivalence() {
    let mut s = String::from("[");
    for i in 0..5000usize {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{k:{}}}", i));
    }
    s.push(']');
    let o = opts();
    let expect = crate::repair_to_string(&s, &o).unwrap();
    let mut buf = Vec::new();
    crate::repair_to_writer_streaming(&s, &o, &mut buf).unwrap();
    let got = String::from_utf8(buf).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&expect).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&got).unwrap();
    assert_eq!(v1, v2);
}
