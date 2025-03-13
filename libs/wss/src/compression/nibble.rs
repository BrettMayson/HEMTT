//! IMA ADPCM-inspired nibble compression for audio PCM data

const PCM_INDEX: [i16; 16] = [
    -8192, -4096, -2048, -1024, -512, -256, -64, 0, 64, 256, 512, 1024, 2048, 4096, 8192,
    0, // Last value is padding for symmetry
];

/// Decompress nibble-compressed data back to PCM samples.
#[allow(clippy::cast_possible_truncation)]
pub fn decompress(data: &[u8]) -> Vec<i16> {
    let mut output = Vec::with_capacity(data.len() * 2);
    let mut delta: i32 = 0;

    for &byte in data {
        // Extract the high nibble (first sample)
        let high_nibble = (byte >> 4) & 0x0F;
        delta += i32::from(PCM_INDEX[high_nibble as usize]);
        // Clamp to i16 range
        output.push(delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);

        // Extract the low nibble (second sample)
        let low_nibble = byte & 0x0F;
        delta += i32::from(PCM_INDEX[low_nibble as usize]);
        // Clamp to i16 range
        output.push(delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
    }

    output
}

/// Compress PCM samples into nibble-compressed data.
pub fn compress(data: &[i16]) -> Vec<u8> {
    let sample_count = data.len();
    let output_size = (sample_count + 1) / 2; // Round up division

    let mut output = Vec::with_capacity(output_size);
    let mut delta: i32 = 0;
    let mut prev_sample: i16 = 0;

    let mut i = 0;
    while i < sample_count {
        let mut byte: u8 = 0;

        // Process first sample into high nibble
        if i < sample_count {
            let target = data[i];

            // Use lookahead to prevent overshoot
            let lookahead = if i + 1 < sample_count {
                data[i + 1]
            } else {
                target
            };
            let trend = (i32::from(target) - i32::from(prev_sample)).signum()
                * (i32::from(lookahead) - i32::from(target)).signum();

            // Find the best nibble with consideration for trend
            let nibble = find_best_nibble_with_trend(delta, target, trend);
            delta += i32::from(PCM_INDEX[nibble as usize]);
            byte |= (nibble << 4) & 0xF0;

            prev_sample = target;
            i += 1;
        }

        // Process second sample into low nibble
        if i < sample_count {
            let target = data[i];

            // Use lookahead to prevent overshoot
            let lookahead = if i + 1 < sample_count {
                data[i + 1]
            } else {
                target
            };
            let trend = (i32::from(target) - i32::from(prev_sample)).signum()
                * (i32::from(lookahead) - i32::from(target)).signum();

            // Find the best nibble with consideration for trend
            let nibble = find_best_nibble_with_trend(delta, target, trend);
            delta += i32::from(PCM_INDEX[nibble as usize]);
            byte |= nibble & 0x0F;

            prev_sample = target;
            i += 1;
        }

        output.push(byte);
    }

    output
}

/// Find the best nibble to represent the desired change from current value to target
/// Considers trend to prevent overshooting
#[allow(clippy::cast_possible_truncation)]
fn find_best_nibble_with_trend(current: i32, target: i16, trend: i32) -> u8 {
    let target_i32 = i32::from(target);

    // If trend is negative (changing direction), prefer smaller steps
    let weight = if trend < 0 { 1.2 } else { 1.0 };

    let mut best_index = 0;
    let mut min_cost = f64::MAX;

    for (i, &delta) in PCM_INDEX.iter().enumerate() {
        let predicted = current + i32::from(delta);
        let error = f64::from((predicted - target_i32).abs());

        // Cost function: penalize larger steps when trend is negative
        let step_size = f64::from(delta.abs());
        let cost = error
            + if trend < 0 {
                weight * step_size / 256.0
            } else {
                0.0
            };

        if cost < min_cost {
            min_cost = cost;
            best_index = i;
        }
    }

    best_index as u8
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

        let compressed = compress(&test_data);
        let decompressed = decompress(&compressed);

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
