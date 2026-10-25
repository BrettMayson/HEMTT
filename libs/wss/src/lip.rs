use std::f32::consts::PI;

use rustfft::{FftPlanner, num_complex::Complex};

use crate::Wss;

impl Wss {
    /// Generates an Arma `.lip` file from the decoded audio.
    pub fn to_lip(&self) -> Result<String, String> {
        crate::lip::generate(
            &self.channel_data,
            self.sample_rate,
            include_bytes!("model-en-mixed.bin"),
        )
    }
}

const MODEL_RECORD_SIZE: usize = 368;
const SAMPLE_RATE: usize = 16_000;
const FRAME_SIZE: usize = 512;
const HOP_SIZE: usize = 256;
const MEL_BANDS: usize = 40;
const COEFFS: usize = 12;
const SILENCE: u8 = 14;

struct Prototype {
    viseme: u8,
    mean: [f32; COEFFS],
    inverse_covariance: [f32; 78],
}

fn load_model(bytes: &[u8]) -> Result<Vec<Prototype>, String> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(MODEL_RECORD_SIZE) {
        return Err("Invalid HeadAudio model size".into());
    }

    let mut model = Vec::new();

    for record in bytes.as_chunks::<MODEL_RECORD_SIZE>().0 {
        let viseme = record[7];

        let mut mean = [0.0; COEFFS];
        for (i, value) in mean.iter_mut().enumerate() {
            let offset = 8 + i * 4;
            *value = f32::from_le_bytes(
                record[offset..offset + 4]
                    .try_into()
                    .map_err(|_| "Failed to read mean value")?,
            );
        }

        let mut inverse_covariance = [0.0; 78];
        for (i, value) in inverse_covariance.iter_mut().enumerate() {
            let offset = 8 + COEFFS * 4 + i * 4;
            *value = f32::from_le_bytes(
                record[offset..offset + 4]
                    .try_into()
                    .map_err(|_| "Failed to read inverse covariance value")?,
            );
        }

        if viseme < 15 {
            model.push(Prototype {
                viseme,
                mean,
                inverse_covariance,
            });
        }
    }

    if model.is_empty() {
        return Err("No valid prototypes in model".into());
    }

    Ok(model)
}

fn classify(features: &[f32; COEFFS], model: &[Prototype]) -> u8 {
    let mut best = SILENCE;
    let mut best_distance = f32::INFINITY;

    for prototype in model {
        let diff: [f32; COEFFS] = std::array::from_fn(|i| features[i] - prototype.mean[i]);

        let mut distance = 0.0;
        let mut index = 0;

        // Symmetric inverse covariance, stored as lower triangle.
        for i in 0..COEFFS {
            for j in 0..=i {
                let value = prototype.inverse_covariance[index];
                if i == j {
                    distance = (value * diff[i]).mul_add(diff[j], distance);
                } else {
                    distance = (2.0 * value * diff[i]).mul_add(diff[j], distance);
                }
                index += 1;
            }
        }

        if prototype.viseme == SILENCE {
            distance /= 1.2;
        }

        if distance < best_distance {
            best_distance = distance;
            best = prototype.viseme;
        }
    }

    best
}

fn extract_mfcc(
    frame: &[f32],
    fft: &std::sync::Arc<dyn rustfft::Fft<f32>>,
    filters: &[[f32; FRAME_SIZE / 2]],
    dct: &[[f32; MEL_BANDS]; COEFFS],
) -> ([f32; COEFFS], f32) {
    let mut spectrum: Vec<Complex<f32>> = frame
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            let window = 0.46f32.mul_add(-(2.0 * PI * i as f32 / 511.0).cos(), 0.54);
            Complex::new(sample * window, 0.0)
        })
        .collect();

    fft.process(&mut spectrum);

    let power: Vec<f32> = spectrum[..FRAME_SIZE / 2]
        .iter()
        .map(|value| value.norm_sqr() / FRAME_SIZE as f32)
        .collect();

    let energy: f32 = power.iter().sum();
    let log_energy = (energy + 1e-10).log10();

    let mel: [f32; MEL_BANDS] = std::array::from_fn(|i| {
        let sum: f32 = power
            .iter()
            .zip(filters[i].iter())
            .map(|(power, filter)| power * filter)
            .sum();

        (sum + 1e-10).log10()
    });

    let features = std::array::from_fn(|i| {
        let coefficient: f32 = dct[i].iter().zip(mel.iter()).map(|(d, m)| d * m).sum();

        let lifter = 11.0f32.mul_add((PI * (i + 1) as f32 / 22.0).sin(), 1.0);
        (coefficient * lifter).tanh()
    });

    (features, log_energy)
}

fn build_filters() -> Vec<[f32; FRAME_SIZE / 2]> {
    let mel = |f: f32| 2595.0 * (1.0 + f / 700.0).log10();
    let inv_mel = |m: f32| 700.0 * (10.0_f32.powf(m / 2595.0) - 1.0);

    let low = mel(30.0);
    let high = mel(7800.0);

    let points: Vec<usize> = (0..MEL_BANDS + 2)
        .map(|i| {
            let m = low + (high - low) * i as f32 / (MEL_BANDS + 1) as f32;
            ((inv_mel(m) / SAMPLE_RATE as f32) * FRAME_SIZE as f32) as usize
        })
        .collect();

    (0..MEL_BANDS)
        .map(|i| {
            let mut filter = [0.0; FRAME_SIZE / 2];
            let left = points[i].min(FRAME_SIZE / 2 - 1);
            let center = points[i + 1].max(left + 1).min(FRAME_SIZE / 2 - 1);
            let right = points[i + 2].max(center + 1).min(FRAME_SIZE / 2);

            for k in left..center {
                filter[k] = (k - left) as f32 / (center - left) as f32;
            }

            for k in center..right {
                filter[k] = (right - k) as f32 / (right - center) as f32;
            }

            filter
        })
        .collect()
}

fn build_dct() -> [[f32; MEL_BANDS]; COEFFS] {
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            (2.0 / MEL_BANDS as f32).sqrt()
                * (PI * (i + 1) as f32 * (j as f32 + 0.5) / MEL_BANDS as f32).cos()
        })
    })
}

// Approximate Oculus -> Arma viseme mapping.
const fn to_arma(viseme: u8) -> u8 {
    match viseme {
        0 => 1,                    // aa
        1 | 8 | 10 | 11 | 12 => 2, // E and similar
        2 | 6 | 13 => 7,           // I / SS / CH
        3 | 4 => 5,                // O / U
        5 => 4,                    // PP
        7 => 6,                    // TH -> v approximation
        9 => 3,                    // FF
        _ => 0,                    // silence
    }
}

/// Generates Arma .lip text from interleaved-independent mono channel data.
///
/// `channels` contains one i16 vector per channel.
/// `sample_rate` is the source sample rate.
/// `model_bytes` is the HeadAudio model-en-mixed.bin contents.
pub fn generate(
    channels: &[Vec<i16>],
    sample_rate: u32,
    model_bytes: &[u8],
) -> Result<String, String> {
    if channels.is_empty() || sample_rate == 0 {
        return Err("Audio must contain at least one channel and a valid sample rate".into());
    }

    let frames = channels.iter().map(Vec::len).min().unwrap_or(0);
    if frames == 0 {
        return Err("Audio contains no samples".into());
    }

    let model = load_model(model_bytes)?;

    // Mix channels to mono.
    let mono: Vec<f32> = (0..frames)
        .map(|i| {
            channels
                .iter()
                .map(|channel| channel[i] as f32 / 32768.0)
                .sum::<f32>()
                / channels.len() as f32
        })
        .collect();

    // Simple linear resampling to 16 kHz.
    // Replace with HeadAudio's polyphase filter for exact compatibility.
    let ratio = sample_rate as f64 / SAMPLE_RATE as f64;
    let output_len = (mono.len() as f64 / ratio).ceil() as usize;
    let audio: Vec<f32> = (0..output_len)
        .map(|i| {
            let position = i as f64 * ratio;
            let left = (position.floor() as usize).min(mono.len() - 1);
            let right = (left + 1).min(mono.len() - 1);
            let fraction = (position - left as f64) as f32;
            mono[right].mul_add(fraction, mono[left] * (1.0 - fraction))
        })
        .collect();

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FRAME_SIZE);
    let filters = build_filters();
    let dct = build_dct();

    // Pre-emphasis.
    let mut emphasized = Vec::with_capacity(audio.len());
    let mut previous = 0.0;
    for sample in audio {
        emphasized.push(0.97f32.mul_add(-previous, sample));
        previous = sample;
    }

    let mut predictions = Vec::new();

    for start in (0..emphasized.len()).step_by(HOP_SIZE) {
        let mut frame = [0.0_f32; FRAME_SIZE];
        let available = (emphasized.len() - start).min(FRAME_SIZE);
        frame[..available].copy_from_slice(&emphasized[start..start + available]);

        let (features, log_energy) = extract_mfcc(&frame, &fft, &filters, &dct);

        let viseme = if log_energy < -4.0 {
            SILENCE
        } else {
            classify(&features, &model)
        };

        predictions.push(viseme);
    }

    let duration = frames as f64 / sample_rate as f64;
    let output_frames = (duration / 0.040).ceil() as usize + 1;

    let mut output = String::from("frame = 0.040\n");

    for frame_index in 0..output_frames {
        let timestamp = frame_index as f64 * 0.040;
        let prediction_index = ((timestamp * SAMPLE_RATE as f64 / HOP_SIZE as f64).round()
            as usize)
            .min(predictions.len().saturating_sub(1));

        let shape = if frame_index == 0 {
            0
        } else {
            to_arma(predictions[prediction_index])
        };

        output.push_str(&format!("{timestamp:.3}, {shape}\n"));
    }

    output.push_str(&format!("{duration:.3}, -1\n"));
    Ok(output)
}
