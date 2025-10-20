use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use std::env;
use std::time::Duration;

fn gen_array_with_spaces(n: usize, spaces: usize) -> String {
    let mut s = String::new();
    s.push('[');
    for i in 0..n {
        if i > 0 {
            s.push_str(&" ".repeat(spaces));
            s.push(',');
        }
        s.push_str(&i.to_string());
    }
    s.push(']');
    s
}

fn gen_object_with_newlines(n: usize, lines: usize) -> String {
    let mut s = String::new();
    s.push('{');
    for i in 0..n {
        if i > 0 {
            s.push(',');
            s.push_str(&"\n".repeat(lines));
        }
        s.push('a');
        s.push_str(&i.to_string());
        s.push(':');
        s.push_str(&(i as i64).to_string());
    }
    s.push('}');
    s
}

fn gen_mixed_comments(size: usize) -> String {
    let mut s = String::new();
    for i in 0..size {
        s.push_str("/*c*/[1,2,3] //x\n");
        s.push_str(&format!("{{k{0}:{0}}}\n", i));
        s.push_str("#y\n\n");
    }
    s
}

fn typical() -> String {
    "{a:1, 'b': 'x', c: /re+/, d: 'he' + 'llo'}".to_string()
}

fn fence_jsonp() -> String {
    "cb(```json\n{a:1}\n```);".to_string()
}

fn unicode_comments() -> String {
    "{'你':/*c*/'好', note: '你' + '好'}".to_string()
}

fn ndjson_lines(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!("{{a:{}}}\n", i));
    }
    s
}

fn gen_flat_object(n: usize) -> String {
    let mut s = String::with_capacity(n * 10);
    s.push('{');
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        s.push('k');
        s.push_str(&i.to_string());
        s.push(':');
        s.push_str(&i.to_string());
    }
    s.push('}');
    s
}

fn gen_array_dense(n: usize) -> String {
    let mut s = String::with_capacity(n * 3);
    s.push('[');
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&i.to_string());
    }
    s.push(']');
    s
}

fn gen_nested_object(depth: usize) -> String {
    let mut s = String::new();
    for _ in 0..depth {
        s.push('{');
        s.push_str("a:");
    }
    s.push_str("{x:1}");
    for _ in 0..depth {
        s.push('}');
    }
    s
}

fn gen_strings_unicode(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str("{text: '你' + '好', i:");
        s.push_str(&i.to_string());
        s.push_str("}\n");
    }
    s
}

fn gen_trailing_commas() -> String {
    let mut s = String::new();
    s.push_str("{a:1,b:2,c:3,}\n");
    s.push_str("[1,2,3,]\n");
    s
}

fn valid_json() -> String {
    r#"{"obj":{"a":1,"b":2,"arr":[1,2,3],"s":"hello","nested":{"x":true,"y":null}}}"#.to_string()
}

fn scale_to_min_bytes(mut s: String) -> String {
    if let Ok(min_bytes_str) = env::var("JR_MIN_BYTES")
        && let Ok(min_bytes) = min_bytes_str.parse::<usize>()
    {
        let size = s.len();
        if size > 0 && size < min_bytes {
            let repeat = min_bytes.div_ceil(size);
            let orig = s.clone();
            for _ in 1..repeat {
                s.push_str(&orig);
            }
        }
    }
    s
}

fn container_llm_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_llm");
    group.sampling_mode(SamplingMode::Flat);
    if let Some(ss) = env::var("JR_SAMPLE_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        group.sample_size(ss.max(1));
    } else {
        group.sample_size(10);
    }
    if let Some(meas) = env::var("JR_MEAS_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.measurement_time(Duration::from_secs(meas));
    } else {
        group.measurement_time(Duration::from_secs(6));
    }
    if let Some(warm) = env::var("JR_WARMUP_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.warm_up_time(Duration::from_secs(warm));
    } else {
        group.warm_up_time(Duration::from_secs(2));
    }

    let opts = llm_json::RepairOptions {
        ensure_ascii: false,
        skip_json_loads: false, // keep validation on for fairness with jsonrepair default
        ..Default::default()
    };

    // Arrays with spaces
    for &spaces in &[64usize, 1024, 8192] {
        let input = scale_to_min_bytes(gen_array_with_spaces(200, spaces));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("array_spaces", spaces), &input, |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts).unwrap();
                std::hint::black_box(out);
            })
        });
    }

    // Objects with many newlines
    for &lines in &[1usize, 8, 64] {
        let input = scale_to_min_bytes(gen_object_with_newlines(200, lines));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("object_newlines", lines),
            &input,
            |b, s| {
                b.iter(|| {
                    let out = llm_json::repair_json(s, &opts).unwrap();
                    std::hint::black_box(out);
                })
            },
        );
    }

    // Mixed comments corpus
    for &rep in &[50usize, 200] {
        let input = scale_to_min_bytes(gen_mixed_comments(rep));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("mixed_comments", rep), &input, |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts).unwrap();
                std::hint::black_box(out);
            })
        });
    }

    // Typical small mixed case
    let input = scale_to_min_bytes(typical());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("typical", "fixed"), &input, |b, s| {
        b.iter(|| {
            let out = llm_json::repair_json(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // Valid JSON baseline (strict)
    let input = scale_to_min_bytes(valid_json());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("valid_json_strict", "fixed"),
        &input,
        |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    // Valid JSON baseline (fastpath: skip validation)
    let mut opts_fast = opts.clone();
    opts_fast.skip_json_loads = true;
    let input = scale_to_min_bytes(valid_json());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("valid_json_fastpath", "fixed"),
        &input,
        |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts_fast).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    // Fence + JSONP
    let input = scale_to_min_bytes(fence_jsonp());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("fence_jsonp", "fixed"), &input, |b, s| {
        b.iter(|| {
            let out = llm_json::repair_json(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // Unicode near comments
    let input = scale_to_min_bytes(unicode_comments());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("unicode_comments", "fixed"),
        &input,
        |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    // NDJSON 500 lines
    let input = scale_to_min_bytes(ndjson_lines(500));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("ndjson", 500), &input, |b, s| {
        b.iter(|| {
            let out = llm_json::repair_json(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // Common realistic corpora (align with jsonrepair container_bench)
    // 1) Large flat object
    let input = scale_to_min_bytes(gen_flat_object(10_000));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("flat_object", 10_000), &input, |b, s| {
        b.iter(|| {
            let out = llm_json::repair_json(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // 2) Dense numeric array
    let input = scale_to_min_bytes(gen_array_dense(100_000));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("array_dense", 100_000), &input, |b, s| {
        b.iter(|| {
            let out = llm_json::repair_json(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // 3) Nested object chain
    let input = scale_to_min_bytes(gen_nested_object(16));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("nested_object", 16), &input, |b, s| {
        b.iter(|| {
            let out = llm_json::repair_json(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // 4) String-heavy with Unicode and concat
    let input = scale_to_min_bytes(gen_strings_unicode(1_000));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("strings_unicode", 1_000),
        &input,
        |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    // 5) Trailing commas
    let input = scale_to_min_bytes(gen_trailing_commas());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("trailing_commas", "fixed"),
        &input,
        |b, s| {
            b.iter(|| {
                let out = llm_json::repair_json(s, &opts).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    group.finish();
}

criterion_group!(benches, container_llm_bench);
criterion_main!(benches);
