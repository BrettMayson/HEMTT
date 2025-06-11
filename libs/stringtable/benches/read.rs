#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use hemtt_stringtable::Project;
use hemtt_workspace::LayerType;

fn criterion_benchmark(c: &mut Criterion) {
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&PathBuf::from("benches"), LayerType::Source)
        .memory()
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("huge_stringtable.xml").unwrap();
    c.bench_function("huge stringtable", |b| {
        b.iter(|| {
            Project::read(source.clone()).unwrap();
        });
    });
    let source = workspace.join("ace_common.xml").unwrap();
    c.bench_function("ace common", |b| {
        b.iter(|| {
            Project::read(source.clone()).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
