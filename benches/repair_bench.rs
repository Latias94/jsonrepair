use criterion::{Criterion, criterion_group, criterion_main};
use jsonrepair::{Options, repair_to_string};

fn bench_repair(c: &mut Criterion) {
    let mut group = c.benchmark_group("repair");
    let cases = vec![
        r#"{a:1}"#,
        r#"// comment
        {"a": 1, /*b*/ "b": 2,}
        "#,
        r#"```json
        {c:3}
        ```
        "#,
        r#"{"text": "The quick brown fox, \n jumps""#,
        r#"undefined"#,
        r#"True False None"#,
    ];
    let opts = Options::default();
    for (i, s) in cases.into_iter().enumerate() {
        group.bench_function(format!("case_{}", i), |b| {
            b.iter(|| {
                let out = repair_to_string(std::hint::black_box(s), &opts).unwrap();
                std::hint::black_box(out);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_repair);
criterion_main!(benches);
