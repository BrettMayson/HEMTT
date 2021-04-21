use std::io::{Error, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{MipMap, PaXType, Paa};

impl Paa {
    pub fn read<I: Seek + Read>(mut input: I) -> Result<Self, Error> {
        if let Some(pax) = PaXType::from_stream(&mut input) {
            let mut paa = Self::new(pax);
            // Read Taggs
            while {
                let mut tagg_sig = [0; 4];
                input.read_exact(&mut tagg_sig)?;
                if let Ok(ts) = std::str::from_utf8(&tagg_sig) {
                    ts == "GGAT"
                } else {
                    false
                }
            } {
                let name = {
                    let mut bytes = [0; 4];
                    input.read_exact(&mut bytes)?;
                    std::str::from_utf8(&bytes).unwrap().to_string()
                };
                paa.taggs.insert(name, {
                    let mut buffer: Box<[u8]> =
                        vec![0; (input.read_u32::<LittleEndian>()?) as usize].into_boxed_slice();
                    input.read_exact(&mut buffer)?;
                    buffer.to_vec()
                });
            }
            // Read MipMaps
            if let Some(offs) = paa.taggs.get("SFFO") {
                for i in 0..(offs.len() / 4) {
                    let mut seek: [u8; 4] = [0; 4];
                    let p = i * 4;
                    seek.clone_from_slice(&offs[p..p + 4]);
                    let seek = u32::from_le_bytes(seek);
                    if seek != 0 {
                        input.seek(SeekFrom::Start(seek as u64))?;
                        paa.maps
                            .push(MipMap::from_stream(paa.format.clone().into(), &mut input)?);
                    }
                }
            }
            Ok(paa)
        } else {
            panic!("unrecognized file");
        }
    }
}
