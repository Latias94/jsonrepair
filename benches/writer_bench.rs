use criterion::{Criterion, Throughput, SamplingMode, black_box, criterion_group, criterion_main};
use std::env;
use std::time::Duration;
use jsonrepair::{Options, repair_to_writer_streaming};

fn gen_large_object(n: usize) -> String {
    let mut s = String::from("{");
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "k{}:{{a:[{}, {}, {}], s:'he'+ 'llo', r:/a\\/b/}}",
            i,
            i,
            i + 1,
            i + 2
        ));
    }
    s.push('}');
    s
}

fn bench_writer(c: &mut Criterion) {
    let opts = Options::default();
    let corpus = gen_large_object(5_000);
    let bytes = corpus.len() as u64;

    let mut g = c.benchmark_group("writer_vs_string");
    g.sampling_mode(SamplingMode::Flat);
    if let Some(ss) = env::var("JR_SAMPLE_SIZE").ok().and_then(|v| v.parse::<usize>().ok()) { g.sample_size(ss.max(1)); } else { g.sample_size(10); }
    if let Some(meas) = env::var("JR_MEAS_SEC").ok().and_then(|v| v.parse::<u64>().ok()) { g.measurement_time(Duration::from_secs(meas)); } else { g.measurement_time(Duration::from_secs(6)); }
    if let Some(warm) = env::var("JR_WARMUP_SEC").ok().and_then(|v| v.parse::<u64>().ok()) { g.warm_up_time(Duration::from_secs(warm)); } else { g.warm_up_time(Duration::from_secs(2)); }
    g.throughput(Throughput::Bytes(bytes));

    g.bench_function("to_string", |b| {
        b.iter(|| {
            let s = jsonrepair::repair_to_string(black_box(&corpus), &opts).unwrap();
            black_box(s);
        })
    });

    g.bench_function("to_writer_streaming", |b| {
        b.iter(|| {
            let mut sink: Vec<u8> = Vec::with_capacity(bytes as usize);
            repair_to_writer_streaming(black_box(&corpus), &opts, &mut sink).unwrap();
            black_box(sink);
        })
    });

    g.finish();
}

criterion_group!(benches, bench_writer);
criterion_main!(benches);
