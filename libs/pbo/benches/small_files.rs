#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::HashMap;

fn criterion_benchmark(c: &mut Criterion) {
    let workspace = hemtt_workspace::Workspace::builder()
        .memory()
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();

    // generate, once, outside of the timed section:
    // 1000 files that are 1 KB each
    // 1000 files that are 10 KB each
    // 1000 files that are 100 KB each
    let mut file_names = Vec::new();
    for i in [1usize, 10, 100] {
        for j in 0..1000 {
            let file_name = format!("file_{i}_{j}.txt");
            let file_path = workspace.join(&file_name).unwrap();
            let content = vec![0u8; i * 1024];
            let mut file = file_path.create_file().unwrap();
            file.write_all(&content).unwrap();
            drop(file);
            file_names.push(file_name);
        }
    }

    c.bench_function("pbo build - small files", |b| {
        b.iter(|| {
            let mut cursors = HashMap::with_capacity(file_names.len());
            for file_name in &file_names {
                let file_path = workspace.join(file_name).unwrap();
                cursors.insert(file_name.clone(), file_path.open_file().unwrap());
            }
            let mut writable_pbo = hemtt_pbo::WritablePbo::new();
            for file_name in &file_names {
                writable_pbo
                    .add_file(file_name.as_str(), cursors.remove(file_name).unwrap())
                    .unwrap();
            }
            let mut output = Vec::new();
            writable_pbo.write(&mut output, true).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
