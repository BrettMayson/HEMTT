use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use hemtt_config::rapify::Derapify;
use hemtt_pbo::ReadablePbo;

use crate::Error;

#[derive(clap::Args)]
pub struct PboUnpackArgs {
    /// PBO file to unpack
    pbo: String,
    /// Directory to unpack to
    output: Option<String>,
    #[arg(long = "derap", short = 'r')]
    /// Unrapifies any rapified files
    derap: bool,
}

/// Execute the unpack command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &PboUnpackArgs) -> Result<(), Error> {
    let pbo_path = PathBuf::from(&args.pbo);
    let mut pbo = ReadablePbo::from(File::open(&pbo_path)?)?;
    let output = args.output.as_ref().map_or_else(
        || {
            pbo_path
                .file_stem()
                .map_or_else(|| PathBuf::from("unpacked_pbo"), PathBuf::from)
        },
        |output| PathBuf::from(&output),
    );
    if output.exists() {
        error!("Output directory already exists");
        return Ok(());
    }
    std::fs::create_dir_all(&output)?;
    for (key, value) in pbo.properties() {
        if key == "prefix" {
            let mut file = File::create(output.join("$PBOPREFIX$"))?;
            file.write_all(value.as_bytes())?;
        } else {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output.join("properties.txt"))?;
            file.write_all(format!("{key}={value}\n").as_bytes())?;
        }
    }
    for header in pbo.files() {
        let path = output.join(header.filename().replace('\\', "/"));
        std::fs::create_dir_all(path.parent().expect("must have parent, just joined"))?;
        let mut out = File::create(&path)?;
        let mut file = pbo
            .file(header.filename())?
            .expect("file must exist if header exists");

        if args.derap {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let mut cursor = std::io::Cursor::new(buffer);
            let mut header = vec![0; 4];
            let Ok(()) = cursor.read_exact(&mut header) else {
                std::io::copy(&mut cursor, &mut out)?;
                continue;
            };
            cursor.set_position(0);
            if header == b"\0raP" || header == b"BLMX" {
                let file_name = path
                    .file_name()
                    .expect("file must have a name")
                    .to_string_lossy();
                let mut raw_out = File::create(match file_name.as_ref() {
                    "config.bin" => path.with_file_name("config.cpp"),
                    "stringtable.bin" => path.with_file_name("stringtable.xml"),
                    _ => path.with_file_name(format!(
                        "{}.derap.{}",
                        path.file_stem()
                            .expect("file must have a stem")
                            .to_string_lossy(),
                        path.extension()
                            .expect("file must have an extension")
                            .to_string_lossy()
                    )),
                })?;
                let derap = match file_name.as_ref() {
                    "stringtable.bin" => {
                        let mut writer = String::new();
                        if let Err(e) = hemtt_stringtable::derapify(
                            pbo_path
                                .file_stem()
                                .expect("must have stem")
                                .to_string_lossy()
                                .to_string(),
                            &mut cursor,
                        )
                        .expect("derap stringtable")
                        .to_writer(&mut writer, true)
                        {
                            error!("failed to write project to stringtable: {}", e);
                            todo!("handle error properly");
                        }
                        writer.as_bytes().to_vec()
                    }
                    _ => hemtt_config::Config::derapify(&mut cursor)?
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                };
                raw_out.write_all(&derap)?;
                cursor.set_position(0);
            }
            out.write_all(&cursor.into_inner())?;
            continue;
        }

        std::io::copy(&mut file, &mut out)?;
    }
    Ok(())
}
