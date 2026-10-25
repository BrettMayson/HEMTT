use hemtt_common::config::Argb8Option;
use hemtt_workspace::{WorkspacePath, reporting::Code};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha1::{Digest, Sha1};

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};
use std::{
    io::{BufReader, Cursor, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use super::Module;

const CACHE_TAGG: &str = "HMTH";

#[derive(Default)]
pub struct Paa {}

impl Module for Paa {
    fn name(&self) -> &'static str {
        "paa"
    }
    fn priority(&self) -> i32 {
        1215 // needs to happen before rapify looks for missing files?
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();

        if !ctx.config().hemtt().paa().enabled() {
            return Ok(report);
        }

        let extensions = ctx.config().hemtt().paa().extensions();
        let allow_argb8 = ctx.config().hemtt().paa().allow_argb8();

        let mut paths = Vec::new();
        for root in ["addons", "optionals"] {
            if !ctx.workspace_path().join(root)?.exists()? {
                continue;
            }
            paths.extend(
                ctx.workspace_path()
                    .join(root)
                    .expect("vfs issue")
                    .walk_dir()
                    .expect("vfs issue")
                    .into_iter()
                    .filter(|p| {
                        p.extension().is_some_and(|e| {
                            extensions.iter().any(|ext| e.eq_ignore_ascii_case(ext))
                        })
                    })
                    .map(|p| {
                        let reader = BufReader::new(p.open_file()?);
                        let image = image::ImageReader::new(reader)
                            .with_guessed_format()?
                            .decode()?;
                        let format = hemtt_paa::PaXType::for_dynamic(&image);
                        if format == hemtt_paa::PaXType::ARGB8 {
                            match allow_argb8 {
                                Argb8Option::Allow => {}
                                Argb8Option::Ask => {
                                    if !crate::utils::paa::confirm_format(format, p.as_str())? {
                                        warn!("Conversion of {} skipped by user", p.as_str());
                                        return Ok(None);
                                    }
                                }
                                Argb8Option::Deny => {
                                    return Ok(None);
                                }
                            }
                        }
                        Ok(Some((p, format)))
                    })
                    .collect::<Result<Vec<_>, Error>>()?
                    .into_iter()
                    .flatten(),
            );
        }
        if paths.is_empty() {
            return Ok(report);
        }
        // Release builds must always produce fresh PAAs, so the cache is only
        // consulted (and kept up-to-date) for non-release builds
        let use_cache = !ctx.config().runtime().is_release();
        let cache_dir = ctx.out_folder().join("paacache");
        if use_cache {
            fs_err::create_dir_all(&cache_dir)?;
        }
        let progress = progress_bar(paths.len() as u64).with_message("Converting");
        let cache_hits = AtomicUsize::new(0);
        let results = paths
            .par_iter()
            .map(|(path, format)| {
                let res = convert_image(path, *format, &cache_dir, use_cache)
                    .map_err(|e| PaaError::code(path, &e));
                if matches!(res, Ok(false)) {
                    cache_hits.fetch_add(1, Ordering::Relaxed);
                }
                progress.inc(1);
                res
            })
            .filter_map(Result::err)
            .collect::<Vec<_>>();
        report.extend(results);
        progress.finish_and_clear();
        let cache_hits = cache_hits.load(Ordering::Relaxed);
        info!(
            "Converted {} images to PAA{}",
            paths.len() - cache_hits,
            if cache_hits > 0 {
                format!(" ({cache_hits} cached)")
            } else {
                String::new()
            }
        );
        Ok(report)
    }
}

/// Convert an image to PAA format
///
/// Returns `Ok(true)` if the image was converted, or `Ok(false)` if an
/// up-to-date PAA was found in the cache and reused instead
fn convert_image(
    in_path: &WorkspacePath,
    format: hemtt_paa::PaXType,
    cache_dir: &std::path::Path,
    use_cache: bool,
) -> Result<bool, Error> {
    trace!("convert_image: {}", in_path.as_str());

    let out_path = in_path.with_extension("paa").expect("vfs err");

    let mut source_bytes = Vec::new();
    BufReader::new(in_path.open_file()?).read_to_end(&mut source_bytes)?;
    let source_hash = content_hash(&source_bytes);

    let cache_path = use_cache.then(|| {
        cache_dir
            .join(path_hash(out_path.as_str()))
            .with_extension("paa")
    });

    if let Some(cache_path) = &cache_path
        && let Some(cached) = read_cached_paa(cache_path, format, &source_hash)
    {
        let mut out_file = out_path.create_file().expect("vfs err");
        out_file.write_all(&cached)?;
        // Remove uncompressed image from virtual FS (so not included in pbo)
        in_path.vfs().remove_file().expect("vfs err");
        return Ok(false);
    }

    let image = image::ImageReader::new(Cursor::new(&source_bytes))
        .with_guessed_format()?
        .decode()?;

    let mut paa = hemtt_paa::Paa::from_dynamic(&image, format)?;
    if use_cache {
        paa.taggs_mut().insert(CACHE_TAGG.to_string(), source_hash);
    }

    let mut cursor = Cursor::new(Vec::new());
    paa.write(&mut cursor)?;
    let buffer = cursor.into_inner();

    let mut out_file = out_path.create_file().expect("vfs err");
    out_file.write_all(&buffer)?;

    if let Some(cache_path) = &cache_path {
        fs_err::write(cache_path, &buffer)?;
    }

    // Remove uncompressed image from virtual FS (so not included in pbo)
    in_path.vfs().remove_file().expect("vfs err");
    Ok(true)
}

/// Read a cached PAA file, returning its bytes if it is still valid for the
/// given format and source image hash
fn read_cached_paa(
    cache_path: &std::path::Path,
    format: hemtt_paa::PaXType,
    source_hash: &[u8],
) -> Option<Vec<u8>> {
    let bytes = fs_err::read(cache_path).ok()?;
    let cached = hemtt_paa::Paa::read(Cursor::new(bytes.as_slice())).ok()?;
    if *cached.format() != format {
        return None;
    }
    if cached.taggs().get(CACHE_TAGG).map(Vec::as_slice) != Some(source_hash) {
        return None;
    }
    Some(bytes)
}

/// Hash the contents of a source image, used to detect if it has changed
fn content_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Hash a workspace path to a stable cache filename
fn path_hash(path: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(path.as_bytes());
    format!("{:x}", hasher.finalize())
}

struct PaaError {
    file: String,
    reason: String,
}

impl Code for PaaError {
    fn ident(&self) -> &'static str {
        "PAA"
    }
    fn message(&self) -> String {
        format!("Failed to convert file to PAA `{}`", self.file)
    }
    fn help(&self) -> Option<String> {
        Some(self.reason.clone())
    }
}

impl PaaError {
    #[must_use]
    pub fn code(path: &WorkspacePath, reason: &Error) -> Arc<dyn Code> {
        Arc::new(Self {
            file: path.as_str().to_string(),
            reason: reason.to_string(),
        })
    }
}
