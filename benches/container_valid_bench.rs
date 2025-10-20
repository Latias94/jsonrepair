use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use jsonrepair::{Options, repair_to_string};
use std::env;
use std::time::Duration;

fn valid_json() -> String {
    // Keep in sync with scripts/py_bench.py and scripts/aggregate_bench.py
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

fn valid_bench(c: &mut Criterion) {
    // Reuse the same group name so aggregator can merge keys uniformly
    let mut group = c.benchmark_group("container_fast_paths");
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

    // valid JSON, ensure_ascii=false (pass-through)
    let opts = Options::default();
    let input = scale_to_min_bytes(valid_json());
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("valid_json", "fixed"), &input, |b, s| {
        b.iter(|| {
            let out = repair_to_string(s, &opts).unwrap();
            std::hint::black_box(out);
        })
    });

    // valid JSON, ensure_ascii=true (ASCII-escaped)
    let opts_ascii = Options {
        ensure_ascii: true,
        ..Default::default()
    };
    let input2 = input.clone();
    group.throughput(Throughput::Bytes(input2.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("valid_json_ensure_ascii", "fixed"),
        &input2,
        |b, s| {
            b.iter(|| {
                let out = repair_to_string(s, &opts_ascii).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    // valid JSON, fastpath (assume valid; skip serde validation)
    let opts_fast = Options {
        assume_valid_json_fastpath: true,
        ..Default::default()
    };
    let input3 = input.clone();
    group.throughput(Throughput::Bytes(input3.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("valid_json_fastpath", "fixed"),
        &input3,
        |b, s| {
            b.iter(|| {
                let out = repair_to_string(s, &opts_fast).unwrap();
                std::hint::black_box(out);
            })
        },
    );

    group.finish();
}

criterion_group!(benches, valid_bench);
criterion_main!(benches);
