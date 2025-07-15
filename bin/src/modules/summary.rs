use crate::{modules::Module, utils::bytes_to_human_readable};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct Summary;
impl Summary {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Drop for Summary {
    fn drop(&mut self) {}
}

impl Module for Summary {
    fn name(&self) -> &'static str {
        "summary"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn post_build(
        &self,
        ctx: &crate::context::Context,
    ) -> Result<crate::report::Report, crate::Error> {
        // give a build summary, sizes of pbos, copied files, etc.
        let report = crate::report::Report::new();

        // Load the last build summary if it exists
        let last_build_path = ctx.out_folder().join("last_build.hsb");
        let last_build = if last_build_path.exists() {
            let mut file = std::fs::File::open(&last_build_path).map_err(|e| {
                crate::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to open last build summary: {e}"),
                ))
            })?;
            SummaryInfo::read(&mut file).map_err(|e| {
                crate::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read last build summary: {e}"),
                ))
            })?
        } else {
            SummaryInfo::default()
        };

        let mut files = Vec::new();
        let mut pbos = Vec::new();

        WalkDir::new(ctx.build_folder().expect("build folder exists"))
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().is_file() {
                    let size = entry.metadata().ok()?.len();
                    match entry.path().extension() {
                        Some(ext) if ext == "pbo" => {
                            pbos.push((entry.file_name().to_string_lossy().to_string(), size));
                            return Some(());
                        }
                        _ => {
                            files.push((entry.file_name().to_string_lossy().to_string(), size));
                            return Some(());
                        }
                    }
                }
                None
            })
            .count();

        let pbo_size = pbos.iter().map(|(_, size)| *size).sum::<u64>();
        let file_size = files.iter().map(|(_, size)| *size).sum::<u64>();
        let total_size = pbo_size + file_size;
        let size_diff = total_size.abs_diff(last_build.last.size);

        println!("Build Summary:");
        println!("  PBOs  : {}", bytes_to_human_readable(pbo_size));
        println!("  Files : {}", bytes_to_human_readable(file_size));
        println!(
            "  Total : {}{}",
            bytes_to_human_readable(total_size),
            if total_size == last_build.last.size || last_build.last.size == 0 {
                String::new()
            } else {
                format!(
                    " ({}{} from last build)",
                    if total_size > last_build.last.size {
                        "↑"
                    } else {
                        "↓"
                    },
                    bytes_to_human_readable(size_diff)
                )
            }
        );
        println!();

        let summary_info = SummaryInfo {
            last: LastBuild {
                size: total_size,
                pbos,
                files,
            },
        };
        summary_info.write(&mut std::fs::File::create(last_build_path)?)?;

        Ok(report)
    }
}

#[derive(Debug, Default)]
struct SummaryInfo {
    last: LastBuild,
}

impl SummaryInfo {
    pub fn write<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        // Write the version
        writer.write_u32::<LittleEndian>(1u32)?;
        // Write the last build summary
        self.last.write(&mut writer)?;
        Ok(())
    }

    pub fn read<R: std::io::Read>(mut reader: R) -> std::io::Result<Self> {
        // Read the version
        let version = reader.read_u32::<LittleEndian>()?;
        if version != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported version",
            ));
        }
        let dev = LastBuild::read(&mut reader)?;
        Ok(Self { last: dev })
    }
}

#[derive(Debug, Default)]
struct LastBuild {
    size: u64,
    pbos: Vec<(String, u64)>,
    files: Vec<(String, u64)>,
}

impl LastBuild {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "u32 is sufficient for file count and name length"
    )]
    pub fn write<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        // Write the version
        writer.write_u32::<LittleEndian>(1u32)?;
        // Write the size
        writer.write_u64::<LittleEndian>(self.size)?;
        // Write the number of PBOs
        writer.write_u32::<LittleEndian>(self.pbos.len() as u32)?;
        // Write each PBO name and size
        for (name, size) in &self.pbos {
            let len = name.len() as u32;
            writer.write_u32::<LittleEndian>(len)?;
            writer.write_all(name.as_bytes())?;
            writer.write_u64::<LittleEndian>(*size)?;
        }
        // Write the number of files
        writer.write_u32::<LittleEndian>(self.files.len() as u32)?;
        // Write each file name and size
        for (name, size) in &self.files {
            let len = name.len() as u32;
            writer.write_u32::<LittleEndian>(len)?;
            writer.write_all(name.as_bytes())?;
            writer.write_u64::<LittleEndian>(*size)?;
        }
        Ok(())
    }

    pub fn read<R: std::io::Read>(mut reader: R) -> std::io::Result<Self> {
        // Read the version
        let version = reader.read_u32::<LittleEndian>()?;
        if version != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported version",
            ));
        }
        // Read the size
        let size = reader.read_u64::<LittleEndian>()?;
        // Read the number of PBOs
        let pbo_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut pbos = Vec::with_capacity(pbo_count);
        for _ in 0..pbo_count {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut name_bytes = vec![0; len];
            reader.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in PBO name")
            })?;
            let size = reader.read_u64::<LittleEndian>()?;
            pbos.push((name, size));
        }
        // Read the number of files
        let file_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut files = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut name_bytes = vec![0; len];
            reader.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid UTF-8 in file name",
                )
            })?;
            let size = reader.read_u64::<LittleEndian>()?;
            files.push((name, size));
        }
        Ok(Self { size, pbos, files })
    }
}
