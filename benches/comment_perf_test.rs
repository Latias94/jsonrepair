use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use jsonrepair::{repair_to_string, Options};
use std::hint::black_box;

fn gen_test_cases() -> Vec<(&'static str, String)> {
    vec![
        ("only_whitespace", " \t\n\r ".repeat(100)),
        ("only_line_comments", "//comment\n".repeat(100)),
        ("only_block_comments", "/*comment*/".repeat(100)),
        ("mixed_simple", "/*c*/[1,2,3] //x\n{k:1}\n#y\n\n".repeat(50)),
        ("deep_nesting", "[".repeat(10) + "1" + &"]".repeat(10)),
    ]
}

fn benchmark_comments(c: &mut Criterion) {
    let mut group = c.benchmark_group("comment_performance");
    let opts = Options::default();

    for (name, input) in gen_test_cases() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, s| {
            b.iter(|| {
                let out = repair_to_string(black_box(s), &opts).unwrap();
                black_box(out);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_comments);
criterion_main!(benches);

