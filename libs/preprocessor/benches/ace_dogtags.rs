use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("preprocess - ace dogtags", |b| {
        b.iter(|| {
            let workspace = hemtt_common::workspace::Workspace::builder()
                .physical(&PathBuf::from("benches"))
                .memory()
                .finish()
                .unwrap();
            let source = workspace.join("ace_dogtags.hpp").unwrap();
            hemtt_preprocessor::Processed::new(&source).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
