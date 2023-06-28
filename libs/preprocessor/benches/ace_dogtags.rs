use criterion::{criterion_group, criterion_main, Criterion};
use hemtt_preprocessor::{preprocess_file, LocalResolver, Processed};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("preprocess - ace dogtags", |b| {
        b.iter(|| {
            let resolver = LocalResolver::default();
            let tokens = preprocess_file("benches/ace_dogtags.hpp", &resolver).unwrap();
            let _ = Processed::from_tokens(&resolver, tokens);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
