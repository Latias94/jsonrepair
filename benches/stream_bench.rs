use criterion::{Criterion, SamplingMode, black_box, criterion_group, criterion_main};
use std::env;
use std::time::Duration;
use jsonrepair::{Options, StreamRepairer};

fn bench_stream(c: &mut Criterion) {
    let mut group = c.benchmark_group("stream");
    group.sampling_mode(SamplingMode::Flat);
    if let Some(ss) = env::var("JR_SAMPLE_SIZE").ok().and_then(|v| v.parse::<usize>().ok()) { group.sample_size(ss.max(1)); } else { group.sample_size(10); }
    if let Some(meas) = env::var("JR_MEAS_SEC").ok().and_then(|v| v.parse::<u64>().ok()) { group.measurement_time(Duration::from_secs(meas)); } else { group.measurement_time(Duration::from_secs(6)); }
    if let Some(warm) = env::var("JR_WARMUP_SEC").ok().and_then(|v| v.parse::<u64>().ok()) { group.warm_up_time(Duration::from_secs(warm)); } else { group.warm_up_time(Duration::from_secs(2)); }

    group.bench_function("ndjson_1000_lines", |b| {
        b.iter(|| {
            let mut r = StreamRepairer::new(Options::default());
            let mut total = 0usize;
            for i in 0..1000 {
                let s = if i % 2 == 0 { "{a:1}\n" } else { "{b:2}\n" };
                let out = r.push(black_box(s)).unwrap();
                total += out.len();
            }
            let tail = r.flush().unwrap();
            total += tail.len();
            black_box(total);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_stream);
criterion_main!(benches);
