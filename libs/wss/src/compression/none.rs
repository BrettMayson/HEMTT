pub fn decompress(data: &[u8], channels: u16) -> Vec<Vec<i16>> {
    let mut output = Vec::new();
    let points = data
        .chunks(2)
        .map(|chunk| {
            let mut buffer = [0; 2];
            buffer.copy_from_slice(chunk);
            i16::from_le_bytes(buffer)
        })
        .collect::<Vec<i16>>();
    for j in 0..channels {
        let mut channel = Vec::new();
        for i in 0..points.len() / channels as usize {
            // every channel is interleaved
            // A,B,A,B,A,B
            channel.push(points[i * channels as usize + j as usize]);
        }
        output.push(channel);
    }
    output
}

pub fn compress(data: &[Vec<i16>]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * 2);
    for i in 0..data[0].len() {
        for sample in data {
            let sample = sample[i];
            output.extend_from_slice(&sample.to_le_bytes());
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_channel() {
        let data = vec![vec![10, 1, 2, 3, 4, 5, 6, 7, 8, 9]];
        let compressed = compress(&data);
        let decompressed = decompress(&compressed, 1);

        assert_eq!(data, decompressed);
    }

    #[test]
    fn two_channels() {
        let data = vec![
            vec![10, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![20, 11, 12, 13, 14, 15, 16, 17, 18, 19],
        ];
        let compressed = compress(&data);
        let decompressed = decompress(&compressed, 2);

        assert_eq!(data, decompressed);
    }
}
