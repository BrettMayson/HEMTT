#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use criterion::{Criterion, criterion_group, criterion_main};
use hemtt_paa::{PaXType, Paa};

fn criterion_benchmark(c: &mut Criterion) {
    let image = image::open("tests/baer.png").unwrap();

    c.bench_function("paa from_dynamic - baer dxt5", |b| {
        b.iter(|| {
            let paa = Paa::from_dynamic(&image, PaXType::DXT5).unwrap();
            std::hint::black_box(paa);
        });
    });

    let paa = Paa::from_dynamic(&image, PaXType::DXT5).unwrap();

    c.bench_function("paa write - baer dxt5", |b| {
        b.iter(|| {
            let mut output = Vec::new();
            paa.write(&mut Cursor::new(&mut output)).unwrap();
            std::hint::black_box(output);
        });
    });

    let mut bytes = Vec::new();
    paa.write(&mut Cursor::new(&mut bytes)).unwrap();

    c.bench_function("paa read - baer dxt5", |b| {
        b.iter(|| {
            let read = Paa::read(Cursor::new(&bytes)).unwrap();
            std::hint::black_box(read);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
