use std::{
    io::{Read, Seek},
    num::NonZero,
};

use vorbis_rs::{VorbisDecoder, VorbisEncoderBuilder};

use crate::{Compression, Error, Wss};

impl Wss {
    #[allow(clippy::cast_possible_truncation)]
    /// Convert the WSS file to an OGG file.
    ///
    /// # Errors
    /// [`Error::Ogg`] if an error occurs while encoding the OGG file.
    ///
    /// # Panics
    /// If the sample rate or number of channels is zero.
    pub fn to_ogg(&self) -> Result<Vec<u8>, Error> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let mut enc = VorbisEncoderBuilder::new(
            NonZero::new(self.sample_rate()).expect("Invalid sample rate"),
            NonZero::new(self.channels() as u8).expect("Invalid number of channels"),
            &mut cursor,
        )?
        .build()?;

        // for sample in samples.chunks(self.channels() as usize * 16) {
        //     let sample: Vec<_> = sample
        //         .iter()
        //         .map(|s| f32::from(*s) / f32::from(i16::MAX))
        //         .collect();
        //     let sample: Vec<Vec<_>> = sample.chunks(self.channels() as usize).fold(
        //         vec![Vec::new(); self.channels() as usize],
        //         |mut acc, s| {
        //             for (i, s) in s.iter().enumerate() {
        //                 acc[i].push(*s);
        //             }
        //             acc
        //         },
        //     );
        //     enc.encode_audio_block(sample)?;
        // }

        for i in 0..self.channel_data[0].len() {
            let mut sample = Vec::new();
            for channel in &self.channel_data {
                sample.push(vec![f32::from(channel[i]) / f32::from(i16::MAX)]);
            }
            enc.encode_audio_block(sample)?;
        }

        enc.finish()?;

        Ok(cursor.into_inner())
    }

    /// Create a new WSS file from a OGG file.
    ///
    /// # Errors
    /// [`Error::Ogg`] if an error occurs while reading the OGG file.
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_ogg<R: Read + Seek>(ogg: R) -> Result<Self, Error> {
        let mut srr = VorbisDecoder::new(ogg)?;
        let mut channel_data = Vec::new();
        let mut first = true;
        while let Some(pck) = srr.decode_audio_block()? {
            for (i, s) in pck.samples().iter().enumerate() {
                if first {
                    channel_data.push(Vec::new());
                }
                channel_data[i].extend(
                    s.iter()
                        .map(|s| (s.clamp(-1.0, 1.0) * 32_767.5).floor() as i16),
                );
            }
            first = false;
        }

        let sample_rate = srr.sampling_frequency().get();
        let channels = u16::from(srr.channels().get());
        Ok(Self {
            compression: Compression::None,
            format: 1,
            channels,
            sample_rate,
            bytes_per_second: sample_rate * u32::from(channels) * 2,
            block_align: channels * 2,
            bits_per_sample: 16,
            output_size: 0,
            channel_data,
        })
    }
}
