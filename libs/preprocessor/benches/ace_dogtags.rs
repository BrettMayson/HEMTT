use criterion::{criterion_group, criterion_main, Criterion};
use hemtt_preprocessor::{preprocess_file, Resolver};
use vfs::PhysicalFS;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("preprocess - ace dogtags", |b| {
        b.iter(|| {
            let vfs = PhysicalFS::new("benches/").into();
            let resolver = Resolver::new(&vfs, Default::default());
            let _ = preprocess_file(&vfs.join("ace_dogtags.hpp").unwrap(), &resolver).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
