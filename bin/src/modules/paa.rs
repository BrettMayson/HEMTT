use hemtt_common::config::Argb8Option;
use hemtt_workspace::{WorkspacePath, reporting::Code};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};
use std::{io::BufReader, sync::Arc};

use super::Module;

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
                    }) // CTX->options here for what file types
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
        let progress = progress_bar(paths.len() as u64).with_message("Converting");
        let results = paths
            .par_iter()
            .map(|(path, format)| {
                let res = convert_image(path, *format).map_err(|e| PaaError::code(path, &e));
                progress.inc(1);
                res
            })
            .filter_map(Result::err)
            .collect::<Vec<_>>();
        report.extend(results);
        progress.finish_and_clear();
        info!("Converted {} images to PAA", paths.len());
        Ok(report)
    }
}

fn convert_image(in_path: &WorkspacePath, format: hemtt_paa::PaXType) -> Result<bool, Error> {
    trace!("convert_image: {}", in_path.as_str());

    let out_path = in_path.with_extension("paa").expect("vfs err");
    let mut out_file = out_path.create_file().expect("vfs err");

    let reader = BufReader::new(in_path.open_file()?);
    let image = image::ImageReader::new(reader)
        .with_guessed_format()?
        .decode()?;

    let paa = hemtt_paa::Paa::from_dynamic(&image, format)?;
    paa.write(&mut out_file)?;

    // Remove uncompressed image from virtual FS (so not included in pbo)
    in_path.vfs().remove_file().expect("vfs err");
    Ok(true)
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
