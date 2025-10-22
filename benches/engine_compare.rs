use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use jsonrepair::{Options, StreamRepairer, options::EngineKind, repair_to_string};
use std::env;
use std::time::Duration;

fn bench_engine_nonstream(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_compare_nonstream");
    group.sampling_mode(SamplingMode::Flat);
    if let Some(ss) = env::var("JR_SAMPLE_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        group.sample_size(ss.max(1));
    }
    if let Some(meas) = env::var("JR_MEAS_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.measurement_time(Duration::from_secs(meas));
    }
    if let Some(warm) = env::var("JR_WARMUP_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.warm_up_time(Duration::from_secs(warm));
    }

    let cases: Vec<(&str, String)> = vec![
        ("small_obj", r#"// c\n{"a": 1, /*b*/ b:2,}"#.to_string()),
        ("fenced_json", "```json\n{a:1}\n```\ntrailing".to_string()),
        ("jsonp_mixed", "cb( {x:1,} );\n{y:2}".to_string()),
        ("concat_strings", "'a' + \"b\" + /*c*/ ' c'".to_string()),
        (
            "numbers_mixed",
            "[.25, 1., 1e, 007, 1/3, 10-20]".to_string(),
        ),
    ];

    for (name, input) in cases {
        group.throughput(Throughput::Bytes(input.len() as u64));

        // Recursive
        let mut opts_rec = Options::default();
        opts_rec.engine = EngineKind::Recursive;
        group.bench_with_input(
            BenchmarkId::new(format!("{}", name), "rec"),
            &input,
            |b, s| {
                b.iter(|| {
                    let out = repair_to_string(std::hint::black_box(s), &opts_rec).unwrap();
                    std::hint::black_box(out);
                })
            },
        );

        // LLM (requires --features llm-compat to be effective)
        let mut opts_llm = Options::default();
        opts_llm.engine = EngineKind::LlmCompat;
        group.bench_with_input(
            BenchmarkId::new(format!("{}", name), "llm"),
            &input,
            |b, s| {
                b.iter(|| {
                    let out = repair_to_string(std::hint::black_box(s), &opts_llm).unwrap();
                    std::hint::black_box(out);
                })
            },
        );
    }

    group.finish();
}

fn bench_engine_stream(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_compare_stream");
    group.sampling_mode(SamplingMode::Flat);
    if let Some(ss) = env::var("JR_SAMPLE_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        group.sample_size(ss.max(1));
    }
    if let Some(meas) = env::var("JR_MEAS_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.measurement_time(Duration::from_secs(meas));
    }
    if let Some(warm) = env::var("JR_WARMUP_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        group.warm_up_time(Duration::from_secs(warm));
    }

    // Build an NDJSON corpus with mixed content
    let mut corpus = String::new();
    for i in 0..2000usize {
        if i % 5 == 0 {
            corpus.push_str("// c\n");
        }
        let line = match i % 4 {
            0 => "{a:1, b:2,}",
            1 => "[1, 2, 3,]",
            2 => "'a' + \"b\"",
            _ => "{n:.25, m:1., e:1e}",
        };
        corpus.push_str(line);
        corpus.push('\n');
    }
    group.throughput(Throughput::Bytes(corpus.len() as u64));

    // Recursive stream
    let mut opts_rec = Options::default();
    opts_rec.engine = EngineKind::Recursive;
    group.bench_function(BenchmarkId::new("ndjson", "rec"), |b| {
        b.iter(|| {
            let mut r = StreamRepairer::new(opts_rec.clone());
            let mut total = 0usize;
            // simple chunking
            for ch in corpus.as_bytes().chunks(4096) {
                let s = std::str::from_utf8(ch).unwrap();
                if let Some(out) = r.push(s).unwrap() {
                    total += out.len();
                }
            }
            if let Some(tail) = r.flush().unwrap() {
                total += tail.len();
            }
            std::hint::black_box(total);
        })
    });

    // LLM stream
    let mut opts_llm = Options::default();
    opts_llm.engine = EngineKind::LlmCompat;
    group.bench_function(BenchmarkId::new("ndjson", "llm"), |b| {
        b.iter(|| {
            let mut r = StreamRepairer::new(opts_llm.clone());
            let mut total = 0usize;
            for ch in corpus.as_bytes().chunks(4096) {
                let s = std::str::from_utf8(ch).unwrap();
                if let Some(out) = r.push(s).unwrap() {
                    total += out.len();
                }
            }
            if let Some(tail) = r.flush().unwrap() {
                total += tail.len();
            }
            std::hint::black_box(total);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_engine_nonstream, bench_engine_stream);
criterion_main!(benches);
