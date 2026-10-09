#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use rand::Rng;

/// Typical uncompressed texture-ish payload: mostly random noise
fn random_data(len: usize) -> Vec<u8> {
    let mut rng = rand::rng();
    (0..len).map(|_| rng.random()).collect()
}

/// Highly repetitive data, representative of flat-color texture regions
fn repetitive_data(len: usize) -> Vec<u8> {
    let pattern = [0u8, 1, 2, 3, 4, 5, 6, 7];
    (0..len).map(|i| pattern[i % pattern.len()]).collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    let random_64k = random_data(64 * 1024);
    let repetitive_64k = repetitive_data(64 * 1024);

    c.bench_function("lz77 compress - random 64k", |b| {
        b.iter(|| {
            let mut out = vec![0u8; random_64k.len() * 2];
            let size = hemtt_lzo::lz77::compress(&random_64k, &mut out).unwrap();
            std::hint::black_box(size);
        });
    });

    c.bench_function("lz77 compress - repetitive 64k", |b| {
        b.iter(|| {
            let mut out = vec![0u8; repetitive_64k.len() * 2];
            let size = hemtt_lzo::lz77::compress(&repetitive_64k, &mut out).unwrap();
            std::hint::black_box(size);
        });
    });

    let mut lz77_compressed = vec![0u8; repetitive_64k.len() * 2];
    let lz77_size = hemtt_lzo::lz77::compress(&repetitive_64k, &mut lz77_compressed).unwrap();
    lz77_compressed.truncate(lz77_size);

    c.bench_function("lz77 decompress - repetitive 64k", |b| {
        b.iter(|| {
            let mut out = vec![0u8; repetitive_64k.len()];
            let size = hemtt_lzo::lz77::decompress(&lz77_compressed, &mut out).unwrap();
            std::hint::black_box(size);
        });
    });

    c.bench_function("lzss compress - random 64k", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(hemtt_lzo::lzss::worst_compress(random_64k.len()));
            hemtt_lzo::lzss::compress(&random_64k, &mut out).unwrap();
            std::hint::black_box(&out);
        });
    });

    c.bench_function("lzss compress - repetitive 64k", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(hemtt_lzo::lzss::worst_compress(repetitive_64k.len()));
            hemtt_lzo::lzss::compress(&repetitive_64k, &mut out).unwrap();
            std::hint::black_box(&out);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
