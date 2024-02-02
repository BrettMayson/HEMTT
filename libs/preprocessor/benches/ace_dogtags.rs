use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use hemtt_common::workspace::LayerType;

fn criterion_benchmark(c: &mut Criterion) {
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&PathBuf::from("benches"), LayerType::Source)
        .memory()
        .finish(None)
        .unwrap();
    let source = workspace.join("ace_dogtags.hpp").unwrap();
    c.bench_function("preprocess - ace dogtags", |b| {
        b.iter(|| {
            hemtt_preprocessor::Processor::run(&source).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
