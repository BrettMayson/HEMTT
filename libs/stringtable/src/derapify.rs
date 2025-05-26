use byteorder::{LittleEndian, ReadBytesExt};
use hemtt_common::io::ReadExt;

use crate::{
    Key, Package,
    project::{InnerProject, WriteableProject},
};

#[allow(clippy::cast_sign_loss)]
/// Converts a stringtable.bin to a `WriteableProject`
///
/// # Errors
/// Returns an error if the input is not a valid derapified project or if reading fails.
pub fn derapify<I: std::io::Read + std::io::Seek>(
    package: String,
    input: &mut I,
) -> Result<WriteableProject, String> {
    let mut header = vec![0; 4];
    input.read_exact(&mut header).map_err(|e| e.to_string())?;
    if &header != b"BLMX" {
        return Err("Invalid derapified project".to_string());
    }

    let language_count = input
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let languages = (0..language_count)
        .map(|_| input.read_cstring().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut offsets = Vec::with_capacity(language_count);
    let _ = input.read_i32::<LittleEndian>();
    for _ in 0..language_count {
        offsets.push(
            input
                .read_i32::<LittleEndian>()
                .map_err(|e| e.to_string())?,
        );
    }

    let key_count = input
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let keys = (0..key_count)
        .map(|_| input.read_cstring().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut translations = vec![Vec::new(); language_count];
    for (i, offset) in offsets.into_iter().enumerate() {
        input
            .seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|e| e.to_string())?;
        let _ = input
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        translations[i] = (0..key_count)
            .map(|_| input.read_cstring().map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
    }

    let mut package = Package::new(package);

    for (i, id) in keys.iter().enumerate() {
        let mut key = Key::new(id.to_owned());
        let english = translations[0].get(i).cloned().unwrap_or_default();
        for l in 0..language_count {
            let language = languages[l].to_lowercase();
            let translation = translations[l].get(i).cloned().unwrap_or_default();
            if !translation.is_empty() && (l == 0 || translation != english) {
                key.set(&language, translation);
            }
        }
        package.add_key(key);
    }

    Ok(WriteableProject::new(
        InnerProject::new(package.name().to_owned(), vec![package]),
        vec![],
    ))
}
