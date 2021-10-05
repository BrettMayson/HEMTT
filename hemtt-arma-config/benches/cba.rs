use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cba_common_macros", |b| {
        b.iter(|| {
            let config = hemtt_arma_config::preprocess(
                hemtt_arma_config::tokenize(
                    &std::fs::read_to_string("tests/cba/script_macros_common.hpp").unwrap(),
                    "tests/cba/script_macros_common.hpp",
                )
                .unwrap(),
                ".",
                hemtt_arma_config::resolver::Basic,
            )
            .unwrap();
            let simplified = hemtt_arma_config::simplify::Config::from_ast(
                hemtt_arma_config::parse(&hemtt_arma_config::render(config).export(), "test")
                    .unwrap(),
            )
            .unwrap();
            let mut buf = Vec::new();
            simplified.write_rapified(&mut buf).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
