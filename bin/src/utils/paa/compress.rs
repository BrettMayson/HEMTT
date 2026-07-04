use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use crate::utils::bytes_to_human_readable;
use crate::Error;

#[derive(clap::Args)]
/// Compress all PAA textures in the project to save space
///
/// This utility walks through the project, finds all PAA files, and compresses
/// any mipmaps that benefit from compression. Each mipmap is analyzed individually,
/// and compression is only applied if the compressed size is smaller than the original.
///
/// A summary is printed at the end showing total space saved.
pub struct Command {
    /// Optional: Process only PAA files in this directory (defaults to current directory)
    #[arg(default_value = ".")]
    path: String,

    /// Only show what would be compressed, don't actually compress
    #[arg(short, long)]
    dry_run: bool,
}

#[derive(Default, Debug)]
struct CompressionStats {
    total_files: u64,
    compressed_files: u64,
    total_mipmaps: u64,
    compressed_mipmaps: u64,
    bytes_before: u64,
    bytes_after: u64,
}

impl CompressionStats {
    fn bytes_saved(&self) -> u64 {
        self.bytes_before.saturating_sub(self.bytes_after)
    }

    fn compression_ratio(&self) -> f64 {
        if self.bytes_before == 0 {
            0.0
        } else {
            (self.bytes_saved() as f64 / self.bytes_before as f64) * 100.0
        }
    }
}

/// Execute the compress command
///
/// # Errors
/// [`Error`] if file operations fail
pub fn execute(cmd: &Command) -> Result<(), Error> {
    let root_path = PathBuf::from(&cmd.path);

    if !root_path.exists() {
        error!("Path does not exist: {}", root_path.display());
        return Ok(());
    }

    info!("Scanning for PAA files in: {}", root_path.display());

    let mut stats = CompressionStats::default();
    let mut files = Vec::new();

    // Find all PAA files
    for entry in walkdir::WalkDir::new(&root_path)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if matches!(
            path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase()),
            Some(ext) if ext == "paa" || ext == "pac"
        ) {
            files.push(path.to_path_buf());
        }
    }

    info!("Found {} PAA file(s)", files.len());

    if files.is_empty() {
        println!("No PAA files found.");
        return Ok(());
    }

    // Process each file
    for file_path in files {
        if let Err(e) = process_paa_file(&file_path, cmd.dry_run, &mut stats) {
            warn!("Failed to process {}: {}", file_path.display(), e);
        }
    }

    // Print summary
    println!("\n=== PAA Compression Summary ===");
    println!(
        "Processed {} PAA file(s), compressed {} file(s)",
        stats.total_files, stats.compressed_files
    );
    println!(
        "Compressed {} out of {} mipmap(s)",
        stats.compressed_mipmaps, stats.total_mipmaps
    );
    println!(
        "Original size:     {}",
        bytes_to_human_readable(stats.bytes_before)
    );
    println!(
        "Compressed size:   {}",
        bytes_to_human_readable(stats.bytes_after)
    );
    println!(
        "Space saved:       {} ({:.2}%)",
        bytes_to_human_readable(stats.bytes_saved()),
        stats.compression_ratio()
    );

    if cmd.dry_run {
        println!("\n(Dry run - no files were modified)");
    }

    Ok(())
}

fn process_paa_file(
    file_path: &Path,
    dry_run: bool,
    stats: &mut CompressionStats,
) -> Result<(), Error> {
    stats.total_files += 1;

    let original_size = std::fs::metadata(file_path)?.len();
    stats.bytes_before += original_size;

    debug!("Processing: {}", file_path.display());

    let mut file = fs_err::File::open(file_path)?;
    let original_paa = hemtt_paa::Paa::read(&mut file)?;

    let mut mipmaps_compressed = 0;

    // Create a new PAA with the same format
    let mut new_paa = hemtt_paa::Paa::new(*original_paa.format());

    // Copy taggs (excluding SFFO which will be regenerated on write)
    for (name, data) in original_paa.taggs() {
        if name != "SFFO" {
            new_paa
                .taggs_mut()
                .insert(name.clone(), data.clone());
        }
    }

    // Process each mipmap
    for (mipmap, _) in original_paa.maps() {
        let width = mipmap.width();
        let height = mipmap.height();
        let format = *mipmap.format();
        let is_already_compressed = mipmap.is_compressed();

        stats.total_mipmaps += 1;

        if !is_already_compressed && format.is_dxt() {
            debug!(
                "  Trying to compress mipmap: {}x{} (format: {:?})",
                width, height, format
            );
            
            // Get original mipmap size when serialized
            let mut original_buffer = Cursor::new(Vec::new());
            mipmap.write(&mut original_buffer)?;
            let original_mipmap_size = original_buffer.get_ref().len();
            
            // Try recompressing
            match hemtt_paa::MipMap::from_rgba_image(
                &mipmap.get_image().to_rgba8(),
                format,
            ) {
                Ok(new_mipmap) => {
                    // Get compressed mipmap size when serialized
                    let mut compressed_buffer = Cursor::new(Vec::new());
                    new_mipmap.write(&mut compressed_buffer)?;
                    let compressed_mipmap_size = compressed_buffer.get_ref().len();
                    
                    if compressed_mipmap_size < original_mipmap_size {
                        debug!(
                            "    Compressed: {} → {} (saved {})",
                            bytes_to_human_readable(original_mipmap_size as u64),
                            bytes_to_human_readable(compressed_mipmap_size as u64),
                            bytes_to_human_readable((original_mipmap_size - compressed_mipmap_size) as u64)
                        );
                        new_paa.push_mipmap(new_mipmap);
                        mipmaps_compressed += 1;
                        stats.compressed_mipmaps += 1;
                    } else {
                        debug!(
                            "    Compression not beneficial: {} vs {}",
                            bytes_to_human_readable(original_mipmap_size as u64),
                            bytes_to_human_readable(compressed_mipmap_size as u64)
                        );
                        new_paa.push_mipmap(mipmap.clone());
                    }
                }
                Err(e) => {
                    warn!(
                        "  Failed to recompress mipmap {}x{}: {}",
                        width, height, e
                    );
                    // Keep original if recompression fails
                    new_paa.push_mipmap(mipmap.clone());
                }
            }
        } else {
            new_paa.push_mipmap(mipmap.clone());
        }
    }

    // If we compressed something and not in dry run mode, write it back
    if mipmaps_compressed > 0 && !dry_run {
        debug!("  Writing compressed PAA: {}", file_path.display());
        let mut output = fs_err::File::create(file_path)?;
        new_paa.write(&mut output)?;
        output.flush()?;

        let compressed_size = std::fs::metadata(file_path)?.len();
        stats.bytes_after += compressed_size;
        stats.compressed_files += 1;

        let saved = original_size.saturating_sub(compressed_size);
        if saved > 0 {
            info!(
                "  {} → {} (saved {})",
                bytes_to_human_readable(original_size),
                bytes_to_human_readable(compressed_size),
                bytes_to_human_readable(saved)
            );
        }
    } else if mipmaps_compressed > 0 && dry_run {
        println!(
            "  [DRY RUN] Would compress {} mipmap(s) in: {}",
            mipmaps_compressed,
            file_path.display()
        );
        stats.bytes_after += original_size; // In dry run, size doesn't change
    } else {
        debug!("  No compression needed for: {}", file_path.display());
        stats.bytes_after += original_size;
    }

    Ok(())
}
