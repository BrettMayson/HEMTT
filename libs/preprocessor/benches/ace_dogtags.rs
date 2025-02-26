#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use hemtt_workspace::LayerType;

fn criterion_benchmark(c: &mut Criterion) {
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&PathBuf::from("benches"), LayerType::Source)
        .memory()
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
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
