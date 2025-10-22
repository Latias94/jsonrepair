use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use jsonrepair::{Options, options::EngineKind, repair_to_string};
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
    "{'��':/*c*/'��', note: '��' + '��'}".to_string()
}
fn ndjson_lines(n: usize) -> String {
    (0..n).map(|i| format!("{{a:{}}}\n", i)).collect()
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
        s.push_str("{text: '��' + '��', i:");
        s.push_str(&i.to_string());
        s.push_str("}\n");
    }
    s
}
fn gen_trailing_commas() -> String {
    "{a:1,b:2,c:3,}\n[1,2,3,]\n".to_string()
}
fn valid_json() -> String {
    r#"{"obj":{"a":1,"b":2,"arr":[1,2,3],"s":"hello","nested":{"x":true,"y":null}}}"#.to_string()
}

fn scale_to_min_bytes(mut s: String) -> String {
    let min_bytes = env::var("JR_MIN_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    if min_bytes > 0 {
        let size = s.len();
        if size > 0 && size < min_bytes {
            let repeat = (min_bytes + size - 1) / size;
            let orig = s.clone();
            for _ in 1..repeat {
                s.push_str(&orig);
            }
        }
    }
    s
}

fn container_our_llm(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_our_llm");
    group.sampling_mode(SamplingMode::Flat);
    if let Some(ss) = env::var("JR_SAMPLE_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        group.sample_size(ss.max(10));
    } else {
        group.sample_size(10);
    }
    if let Some(meas) = env::var("JR_MEAS_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.measurement_time(Duration::from_secs(meas));
    } else {
        group.measurement_time(Duration::from_secs(2));
    }
    if let Some(warm) = env::var("JR_WARMUP_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.warm_up_time(Duration::from_secs(warm));
    } else {
        group.warm_up_time(Duration::from_secs(1));
    }

    let mut base = Options::default();
    base.engine = EngineKind::LlmCompat;
    base.ensure_ascii = false;
    base.assume_valid_json_fastpath = false;

    for &spaces in &[64usize, 1024, 8192] {
        let input = scale_to_min_bytes(gen_array_with_spaces(200, spaces));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("array_spaces", spaces), &input, |b, s| {
            b.iter(|| {
                let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
            })
        });
    }
    for &lines in &[1usize, 8, 64] {
        let input = scale_to_min_bytes(gen_object_with_newlines(200, lines));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("object_newlines", lines),
            &input,
            |b, s| {
                b.iter(|| {
                    let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
                })
            },
        );
    }
    for &rep in &[50usize, 200] {
        let input = scale_to_min_bytes(gen_mixed_comments(rep));
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("mixed_comments", rep), &input, |b, s| {
            b.iter(|| {
                let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
            })
        });
    }
    // Typical
    let input = scale_to_min_bytes(typical());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("typical", "fixed"), &input, |b, s| {
        b.iter(|| {
            let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
        })
    });
    // Valid strict
    let input = scale_to_min_bytes(valid_json());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("valid_json_strict", "fixed"),
        &input,
        |b, s| {
            b.iter(|| {
                let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
            })
        },
    );
    // Fence + JSONP
    let input = scale_to_min_bytes(fence_jsonp());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("fence_jsonp", "fixed"), &input, |b, s| {
        b.iter(|| {
            let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
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
                let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
            })
        },
    );
    // NDJSON 500
    let input = scale_to_min_bytes(ndjson_lines(500));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("ndjson", 500), &input, |b, s| {
        b.iter(|| {
            let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
        })
    });
    // Large flat object
    let input = scale_to_min_bytes(gen_flat_object(10_000));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("flat_object", 10_000), &input, |b, s| {
        b.iter(|| {
            let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
        })
    });
    // Dense array
    let input = scale_to_min_bytes(gen_array_dense(100_000));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("array_dense", 100_000), &input, |b, s| {
        b.iter(|| {
            let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
        })
    });
    // Nested object
    let input = scale_to_min_bytes(gen_nested_object(16));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("nested_object", 16), &input, |b, s| {
        b.iter(|| {
            let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
        })
    });
    // Strings Unicode
    let input = scale_to_min_bytes(gen_strings_unicode(1_000));
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("strings_unicode", 1_000),
        &input,
        |b, s| {
            b.iter(|| {
                let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
            })
        },
    );
    // Trailing commas corpus
    let input = scale_to_min_bytes(gen_trailing_commas());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("trailing_commas", "fixed"),
        &input,
        |b, s| {
            b.iter(|| {
                let _ = repair_to_string(std::hint::black_box(s), &base).unwrap();
            })
        },
    );

    group.finish();
}

criterion_group!(benches, container_our_llm);
criterion_main!(benches);
