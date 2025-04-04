//! IMA ADPCM-inspired nibble compression for audio PCM data

const PCM_INDEX: [i16; 15] = [
    -8192, -4096, -2048, -1024, -512, -256, -64, 0, 64, 256, 512, 1024, 2048, 4096, 8192,
];

/// Decompress nibble-compressed data back to PCM samples.
pub fn decompress(data: &[u8], channels: u16) -> Vec<Vec<i16>> {
    let channels = channels as usize;
    let mut working = Vec::with_capacity(channels);
    let mut output = Vec::with_capacity(channels);

    for _ in 0..channels {
        working.push(Vec::new());
    }

    let mut current_channel = 0;
    let mut i = 0;
    while i < data.len() {
        let point = data[i];
        working[current_channel].push(point);
        i += 1;
        current_channel += 1;
        if current_channel == channels {
            current_channel = 0;
        }
    }

    for channel in working {
        output.push(decompress_mono(&channel));
    }

    output
}

#[allow(clippy::cast_possible_truncation)]
fn decompress_mono(data: &[u8]) -> Vec<i16> {
    let mut delta: i32 = 0;
    let mut output = Vec::with_capacity(data.len() * 2);

    for &byte in data {
        // Extract the high nibble (first sample)
        let high_nibble = (byte >> 4) & 0x0F;
        delta += i32::from(PCM_INDEX[high_nibble as usize]);
        output.push(delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);

        // Extract the low nibble (second sample)
        let low_nibble = byte & 0x0F;
        delta += i32::from(PCM_INDEX[low_nibble as usize]);
        output.push(delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
    }

    output
}

pub fn compress(data: &[Vec<i16>]) -> Vec<u8> {
    let data = data
        .iter()
        .map(|channel| compress_mono(channel))
        .collect::<Vec<_>>();
    let mut output = Vec::new();
    for i in 0..data[0].len() {
        for channel in &data {
            output.push(channel[i]);
        }
    }
    output
}

#[allow(clippy::cast_possible_truncation)]
fn compress_mono(data: &[i16]) -> Vec<u8> {
    let output_size = data.len().div_ceil(2); // Round up for odd number of samples
    let mut output = Vec::with_capacity(output_size);

    let mut delta: i32 = 0;
    let mut i = 0;

    while i < data.len() {
        let mut byte: u8 = 0;

        // Process first sample for high nibble
        if i < data.len() {
            let target = i32::from(data[i]);
            let index = find_best_index_for_target(delta, target);
            byte |= (index as u8) << 4;
            delta += i32::from(PCM_INDEX[index]);
            i += 1;
        }

        // Process second sample for low nibble
        if i < data.len() {
            let target = i32::from(data[i]);
            let index = find_best_index_for_target(delta, target);
            byte |= index as u8;
            delta += i32::from(PCM_INDEX[index]);
            i += 1;
        }

        output.push(byte);
    }

    output
}

/// Find the index in `PCM_INDEX` that, when added to delta,
/// gets closest to the target value.
fn find_best_index_for_target(delta: i32, target: i32) -> usize {
    let mut best_index = 0;
    let mut min_error = i32::MAX;

    for (i, &pcm_value) in PCM_INDEX.iter().enumerate() {
        let new_delta = delta + i32::from(pcm_value);
        let error = (target - new_delta).abs();
        if error < min_error {
            min_error = error;
            best_index = i;
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn test_roundtrip() {
        // Create test data with a simple sine wave
        let mut test_data = Vec::with_capacity(1000);
        for i in 0..1000 {
            let value = ((i as f32 * 0.1).sin() * 8000.0) as i16;
            test_data.push(value);
        }

        let compressed = compress(&[test_data.clone()]);
        let decompressed = decompress(&compressed, 1)[0].clone();

        // Check errors
        let mut total_error = 0.0;
        for (&original, &decoded) in test_data.iter().zip(decompressed.iter()) {
            let error = (f32::from(original) - f32::from(decoded)).abs();
            total_error += error;
        }

        let avg_error = total_error / test_data.len() as f32;
        assert!(avg_error < 200.0);
    }
}
