#[allow(clippy::excessive_precision)]
const LN_10: f64 = 2.302_585_092_994_045_684_084_0;
#[allow(clippy::excessive_precision)]
const LOG2: f64 = 1.442_695_040_888_963_407_0;

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

#[allow(clippy::suboptimal_flops)]
fn decompress_mono(data: &[u8]) -> Vec<i16> {
    let magic_number: f64 = (LN_10 * LOG2) / 28.125_740_425_151_72;
    let mut output_data = Vec::with_capacity(data.len());
    let mut last_val: i16 = 0;

    for &byte in data {
        #[allow(clippy::cast_possible_wrap)]
        let byte = byte as i8;
        if byte != 0 {
            let mut as_float = f64::from(byte).abs() * magic_number;
            let rnd = as_float.round();
            as_float = (2.0f64).powf(as_float - rnd) * (2.0f64).powf(rnd);

            if byte < 0 {
                as_float *= -1.0;
            }

            let as_int = as_float.round() + f64::from(last_val);
            #[allow(clippy::cast_possible_truncation)]
            let clamped_int = as_int.round() as i64;

            // Clamp to the short range
            #[allow(clippy::cast_possible_truncation)]
            let clamped_int = clamped_int.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16;

            last_val = clamped_int;
        }
        output_data.push(last_val);
    }

    output_data
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

#[allow(clippy::suboptimal_flops)]
fn compress_mono(data: &[i16]) -> Vec<u8> {
    let magic_number: f64 = (LN_10 * LOG2) / 28.125_740_425_151_72;
    let mut output_data = Vec::with_capacity(data.len());
    let mut last_val: i16 = 0;

    for &value in data {
        let delta = value.wrapping_sub(last_val);

        if delta == 0 {
            output_data.push(0);
            last_val = value;
            continue;
        }

        // For non-zero delta, calculate the appropriate byte value
        let abs_delta = f64::from(delta).abs();

        // Handle very small deltas better to prevent amplification of noise
        let byte_value = if abs_delta < 1.0 {
            0.5
        } else {
            abs_delta.log2() / magic_number
        };

        let signed_byte_value = if delta < 0 { -byte_value } else { byte_value };

        #[allow(clippy::cast_possible_truncation)]
        let byte = signed_byte_value
            .round()
            .clamp(f64::from(i8::MIN), f64::from(i8::MAX)) as i8;

        // Ensure non-zero byte for non-zero delta (but don't amplify tiny differences too much)
        let byte = if byte == 0 {
            if abs_delta < 3.0 {
                // For very small deltas, use minimal change
                if delta > 0 { 1 } else { -1 }
            } else {
                // For larger deltas that still round to zero
                if delta > 0 { 2 } else { -2 }
            }
        } else {
            byte
        };

        #[allow(clippy::cast_sign_loss)]
        output_data.push(byte as u8);

        // Simulate decompression to get the actual value that will be reproduced
        let mut as_float = f64::from(byte).abs() * magic_number;
        let rnd = as_float.round();
        as_float = (2.0f64).powf(as_float - rnd) * (2.0f64).powf(rnd);

        if byte < 0 {
            as_float *= -1.0;
        }

        #[allow(clippy::cast_possible_truncation)]
        {
            let decompressed_delta = as_float.round() as i32;
            last_val = (i32::from(last_val) + decompressed_delta)
                .clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
        }
    }

    output_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn test_roundtrip_accuracy() {
        let mut test_data = Vec::with_capacity(1000);
        for i in 0..1000 {
            let value = (i as f32 * 0.1)
                .sin()
                .mul_add(5000.0, (i as f32 * 0.05).cos() * 2000.0) as i16;
            test_data.push(value);
        }

        let compressed = compress(&[test_data.clone()]);
        let decompressed = decompress(&compressed, 1)[0].clone();

        let mut total_error = 0.0;
        for (&original, &decoded) in test_data.iter().zip(decompressed.iter()) {
            let error = (f32::from(original) - f32::from(decoded)).abs();
            total_error += error;
        }

        let avg_error = total_error / test_data.len() as f32;

        assert!(avg_error < 10.0);
    }
}
