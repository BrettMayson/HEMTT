pub fn decompress(data: &[u8]) -> Vec<i16> {
    let mut output = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let mut buffer = [0; 2];
        buffer.copy_from_slice(&data[i..i + 2]);
        output.push(i16::from_le_bytes(buffer));
        i += 2;
    }
    output
}

pub fn compress(data: &[i16]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * 2);
    for &value in data {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_and_decompress() {
        let data = vec![10, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let compressed = compress(&data);
        let decompressed = decompress(&compressed);

        assert_eq!(data, decompressed);
    }
}
