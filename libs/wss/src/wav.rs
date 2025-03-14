use std::io::Read;

use crate::{Compression, Error, Wss};

impl Wss {
    /// Converts the WSS to a WAV file.
    ///
    /// # Errors
    /// [`Error::Wav`] if an error occurs while writing the WAV file.
    pub fn to_wav(&self) -> Result<Vec<u8>, Error> {
        let mut cursor = std::io::Cursor::new(Vec::new());

        let spec = hound::WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: self.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;

        // let samples = self.compression().decompress(&self.data);

        // for sample in samples {
        //     writer.write_sample(sample)?;
        // }

        for i in 0..self.channel_data[0].len() {
            for channel in &self.channel_data {
                writer.write_sample(channel[i])?;
            }
        }

        drop(writer);

        Ok(cursor.into_inner())
    }

    /// Create a new WSS file from a WAV file.
    ///
    /// # Errors
    /// [`Error::Wav`] if an error occurs while reading the WAV file.
    pub fn from_wav<R: Read>(wav: R) -> Result<Self, Error> {
        let mut reader = hound::WavReader::new(wav)?;
        let channels = reader.spec().channels;
        let sample_rate = reader.spec().sample_rate;
        let bits_per_sample = reader.spec().bits_per_sample;

        let mut data = Vec::new();
        for sample in reader.samples::<i16>() {
            data.push(sample?);
        }
        let mut channel_data = Vec::new();
        for _ in 0..channels {
            channel_data.push(Vec::new());
        }
        for (i, sample) in data.iter().enumerate() {
            channel_data[i % channels as usize].push(*sample);
        }

        Ok(Self {
            compression: Compression::None,
            format: 1,
            channels,
            sample_rate,
            bytes_per_second: u32::from(channels) * u32::from(bits_per_sample) / 8 * sample_rate,
            block_align: channels * bits_per_sample / 8,
            bits_per_sample,
            output_size: 0,
            channel_data,
        })
    }
}
