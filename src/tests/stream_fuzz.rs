use super::*;
// use helper defined in this module directly

fn collect_stream(mut r: crate::StreamRepairer, chunks: &[String]) -> Vec<String> {
    let mut outs = Vec::new();
    for c in chunks {
        if let Some(s) = r.push(c).unwrap() {
            outs.push(s);
        }
    }
    if let Some(tail) = r.flush().unwrap() {
        outs.push(tail);
    }
    outs
}

#[test]
fn st_large_array_container_memchr_jump_integrity_random_chunks() {
    // Build a moderately large array with spaces and comments sprinkled
    let mut src = String::from("[");
    for i in 0..2000usize {
        if i > 0 {
            src.push(',');
        }
        src.push_str(&format!("{}", i));
        if i % 7 == 0 {
            src.push_str(" /*c*/ ");
        }
        if i % 13 == 0 {
            src.push_str("    ");
        }
    }
    src.push_str("]\n");
    let sizes = super::lcg_sizes(1234567, src.chars().count());
    let parts = super::chunk_by_char(&src, &sizes);
    let outs = collect_stream(crate::StreamRepairer::new(Options::default()), &parts);
    let merged: String = outs.concat();
    let v: serde_json::Value = serde_json::from_str(&merged).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 2000);
    assert_eq!(arr[0], serde_json::json!(0));
    assert_eq!(arr[1999], serde_json::json!(1999));
}

#[test]
fn st_large_object_container_memchr_jump_integrity_random_chunks() {
    let mut src = String::from("{");
    for i in 0..1500usize {
        if i > 0 {
            src.push(',');
        }
        src.push_str(&format!("k{}:{}", i, i));
        if i % 11 == 0 {
            src.push_str(" //x\n");
        }
        if i % 17 == 0 {
            src.push_str(" /*y*/ ");
        }
    }
    src.push_str("}\n");
    let sizes = super::lcg_sizes(7654321, src.chars().count());
    let parts = super::chunk_by_char(&src, &sizes);
    let outs = collect_stream(crate::StreamRepairer::new(Options::default()), &parts);
    let merged: String = outs.concat();
    let v: serde_json::Value = serde_json::from_str(&merged).unwrap();
    let obj = v.as_object().expect("object");
    assert_eq!(obj.len(), 1500);
    assert_eq!(obj.get("k0"), Some(&serde_json::json!(0)));
    assert_eq!(obj.get("k1499"), Some(&serde_json::json!(1499)));
}

#[test]
fn st_huge_ndjson_3000_random_chunks() {
    let mut src = String::new();
    for i in 0..3000usize {
        if i % 9 == 0 {
            src.push_str("# c\n");
        }
        src.push_str(&format!("{{i:{}}}\n", i));
    }
    let sizes = super::lcg_sizes(42, src.chars().count());
    let parts = super::chunk_by_char(&src, &sizes);
    let outs = collect_stream(crate::StreamRepairer::new(Options::default()), &parts);
    let merged: String = outs.concat();
    let de = serde_json::Deserializer::from_str(&merged).into_iter::<serde_json::Value>();
    let mut cnt = 0usize;
    for v in de {
        v.unwrap();
        cnt += 1;
    }
    assert_eq!(cnt, 3000);
}

#[test]
fn st_large_jsonp_fenced_unicode_random_chunks() {
    let src = "cb(\n```json\n{a:1}\n```\n)\n{\u{4F60}\u{597D}: '世界'}\n"; // JSONP + fenced + unicode object
    let sizes = super::lcg_sizes(987654321, src.chars().count());
    let parts = super::chunk_by_char(src, &sizes);
    let outs = collect_stream(crate::StreamRepairer::new(Options::default()), &parts);
    assert_eq!(outs.len(), 2);
    let v0: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&outs[1]).unwrap();
    assert_eq!(v0, serde_json::json!({"a":1}));
    assert_eq!(v1, serde_json::json!({"你好":"世界"}));
}

#[test]
fn st_fuzz_random_chunks_small_seed_variations() {
    // light sanity: ensure no panic and at least one value emitted across seeds
    let srcs = vec![
        "{a:1}\n{b:2}\n",
        "# c\n{c:3}\n\n{d:4}\n",
        "```json\n{e:5}\n```\n",
    ];
    for (seed, s) in [1u64, 2, 3, 4, 5]
        .into_iter()
        .zip(srcs.into_iter().cycle().take(5))
    {
        let sizes = super::lcg_sizes(seed, s.chars().count());
        let parts = super::chunk_by_char(s, &sizes);
        let outs = collect_stream(crate::StreamRepairer::new(Options::default()), &parts);
        assert!(!outs.is_empty());
    }
}
