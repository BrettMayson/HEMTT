use std::io::Read;

use crate::{Compression, Error, Wss};

impl Wss {
    /// Create a new WSS file from a MP3 file.
    ///
    /// # Errors
    /// [`Error::Ogg`] if an error occurs while reading the MP3 file.
    pub fn from_mp3<R: Read>(mp3: R) -> Result<Self, Error> {
        Self::from_mp3_with_compression(mp3, Compression::default())
    }

    /// Create a new WSS file from a MP3 file with a specific compression type.
    ///
    /// # Errors
    /// [`Error::Ogg`] if an error occurs while reading the MP3 file.
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_mp3_with_compression<R: Read>(
        mp3: R,
        compression: Compression,
    ) -> Result<Self, Error> {
        let (header, samples) = puremp3::read_mp3(mp3)?;
        let mut data = Vec::new();
        for (left, right) in samples {
            data.push((left.clamp(-1.0, 1.0) * 32_767.5).floor() as i16);
            data.push((right.clamp(-1.0, 1.0) * 32_767.5).floor() as i16);
        }
        let data = compression.compress(&data);
        println!("Sample rate: {:?}", header.sample_rate.hz());
        println!("Bitrate: {:?}", header.bitrate);
        Ok(Self {
            compression,
            format: 1,
            channels: 2,
            sample_rate: header.sample_rate.hz(),
            bytes_per_second: header.sample_rate.hz() * 4,
            block_align: 4,
            bits_per_sample: 16,
            output_size: 0,
            data,
        })
    }
}
