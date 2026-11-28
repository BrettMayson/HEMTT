#![allow(clippy::unwrap_used, clippy::cast_possible_truncation)]

//! Round trips for both compressors over inputs with different shapes,
//! and decompression of damaged input, which must return an error
//! instead of panicking.

use hemtt_lzo::{lz77, lzss};

/// A small deterministic generator, so failures can be reproduced
struct Rng(u64);

impl Rng {
    const fn next(&mut self) -> u64 {
        // xorshift64
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next() as u8).collect()
    }
}

fn corpus() -> Vec<(String, Vec<u8>)> {
    let mut rng = Rng(0x5EED);
    let text =
        b"class CfgPatches { class test_main { units[] = {}; weapons[] = {}; }; };\n".repeat(200);
    let mut corpus = vec![
        ("single byte".to_string(), vec![42]),
        ("two bytes".to_string(), vec![1, 2]),
        ("three equal".to_string(), vec![7, 7, 7]),
        ("zeros".to_string(), vec![0; 10_000]),
        ("spaces".to_string(), vec![b' '; 5_000]),
        ("0xFF".to_string(), vec![0xFF; 5_000]),
        ("text".to_string(), text),
        ("random".to_string(), rng.bytes(20_000)),
        (
            // A repeat further back than lz77's 4095 byte window
            "far repeat".to_string(),
            {
                let block = rng.bytes(5_000);
                [block.clone(), block].concat()
            },
        ),
        (
            // Runs of every length around the max match length
            "runs".to_string(),
            (0..40u8)
                .flat_map(|i| std::iter::repeat_n(i, usize::from(i)))
                .collect(),
        ),
        (
            "low entropy".to_string(),
            (0..30_000).map(|_| (rng.next() % 3) as u8).collect(),
        ),
    ];
    for len in [
        4, 7, 8, 9, 15, 16, 17, 63, 64, 65, 255, 256, 4095, 4096, 4097,
    ] {
        corpus.push((format!("random {len}"), rng.bytes(len)));
    }
    corpus
}

fn lz77_compress(data: &[u8]) -> Vec<u8> {
    // Worst case is one flag byte per 8 raw bytes, plus the checksum
    let mut out = vec![0; data.len() + data.len() / 8 + 16];
    let size = lz77::compress(data, &mut out).unwrap();
    out.truncate(size);
    out
}

fn lzss_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(lzss::worst_compress(data.len()));
    lzss::compress(data, &mut out).unwrap();
    out
}

#[test]
fn lz77_roundtrip() {
    for (name, data) in corpus() {
        let compressed = lz77_compress(&data);
        let mut out = vec![0; data.len()];
        let read =
            lz77::decompress(&compressed, &mut out).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(out, data, "{name}");
        assert_eq!(read, compressed.len(), "{name}: all input is consumed");
    }
}

#[test]
fn lz77_empty() {
    let mut out = [0; 4];
    assert_eq!(lz77::compress(&[], &mut out), Ok(0));
}

#[test]
fn lz77_compresses_repetitive_data() {
    let data = vec![0; 10_000];
    assert!(lz77_compress(&data).len() < data.len() / 4);
}

#[test]
fn lz77_output_too_small() {
    let data = Rng(1).bytes(1000);
    let mut out = vec![0; 100];
    assert!(lz77::compress(&data, &mut out).is_err());
}

#[test]
fn lz77_truncated_input_is_an_error() {
    for (name, data) in corpus() {
        let compressed = lz77_compress(&data);
        // Leave out the checksum and at least one byte of the stream
        for len in (0..compressed.len().saturating_sub(4)).step_by(compressed.len() / 50 + 1) {
            let mut out = vec![0; data.len()];
            assert!(
                lz77::decompress(&compressed[..len], &mut out).is_err(),
                "{name}: truncated to {len} of {} bytes was accepted",
                compressed.len()
            );
        }
    }
}

#[test]
fn lz77_garbage_does_not_panic() {
    let mut rng = Rng(0xBAD);
    for _ in 0..2_000 {
        let len = (rng.next() % 512) as usize;
        let input = rng.bytes(len);
        let out_len = (rng.next() % 2048) as usize;
        let mut out = vec![0; out_len];
        // Any result is fine, as long as it doesn't panic
        let _ = lz77::decompress(&input, &mut out);
    }
}

#[test]
fn lzss_roundtrip() {
    for (name, data) in corpus() {
        let compressed = lzss_compress(&data);
        let mut out = vec![0; data.len()];
        let result = lzss::decompress_to_slice(&compressed, &mut out)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(result, &data[..], "{name}");
    }
}

#[test]
fn lzss_compress_to_slice_matches_compress() {
    for (name, data) in corpus() {
        let mut buffer = vec![0; lzss::worst_compress(data.len())];
        let slice = lzss::compress_to_slice(&data, &mut buffer)
            .unwrap()
            .to_vec();
        assert_eq!(slice, lzss_compress(&data), "{name}");
    }
}

#[test]
fn lzss_output_too_small_is_an_error() {
    let data =
        b"some data that is long enough to compress, some data that is long enough".repeat(10);
    let compressed = lzss_compress(&data);
    let mut out = vec![0; data.len() - 1];
    assert_eq!(
        lzss::decompress_to_slice(&compressed, &mut out).map(|s| s.len()),
        Err(lzss::LzoError::OutputOverrun)
    );
}

#[test]
fn lzss_truncated_input_is_an_error() {
    for (name, data) in corpus() {
        let compressed = lzss_compress(&data);
        for len in (0..compressed.len()).step_by(compressed.len() / 50 + 1) {
            let mut out = vec![0; data.len()];
            assert!(
                lzss::decompress_to_slice(&compressed[..len], &mut out).is_err(),
                "{name}: truncated to {len} of {} bytes was accepted",
                compressed.len()
            );
        }
    }
}

#[test]
fn lzss_garbage_does_not_panic() {
    let mut rng = Rng(0xF00D);
    for _ in 0..2_000 {
        let len = (rng.next() % 512) as usize;
        let input = rng.bytes(len);
        let mut out = vec![0; (rng.next() % 4096) as usize];
        let _ = lzss::decompress_to_slice(&input, &mut out);
    }
}

#[test]
fn lz77_checksum_wraps_on_large_input() {
    // 17M bytes of 0xFF sum to more than u32::MAX, which used to panic in debug builds
    let data = vec![0xFF; 17_000_000];
    let compressed = lz77_compress(&data);
    let mut out = vec![0; data.len()];
    lz77::decompress(&compressed, &mut out).unwrap();
    assert!(out == data);
}
