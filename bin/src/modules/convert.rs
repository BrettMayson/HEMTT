use hemtt_workspace::{WorkspacePath, reporting::Code};
use image::GenericImageView;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};
use std::{io::BufReader, sync::Arc};

use super::Module;

#[derive(Default)]
pub struct Convert {}

impl Module for Convert {
    fn name(&self) -> &'static str {
        "convert"
    }
    fn priority(&self) -> i32 {
        1215 // needs to happen before rapify looks for missing files?
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();

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
                    .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"))), // CTX->options here for what file types
            );
        }
        let progress = progress_bar(paths.len() as u64).with_message("Converting");
        let results = paths
            .par_iter()
            .map(|path| {
                let res = convert_image(path).map_err(|e| ConvertError::code(path, e));
                progress.inc(1);
                res
            })
            .filter_map(Result::err)
            .collect::<Vec<_>>();
        report.extend(results);
        progress.finish_and_clear();
        Ok(report)
    }
}

fn convert_image(in_path: &WorkspacePath) -> Result<(), String> {
    trace!("convert_image: {}", in_path.as_str());

    let out_path = in_path.with_extension("paa").expect("vfs err");
    let mut out_file = out_path.create_file().expect("vfs err");

    let reader = BufReader::new(in_path.open_file().map_err(|e| e.to_string())?);
    let image = image::ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let (width, height) = image.dimensions();
    if !height.is_power_of_two() || !width.is_power_of_two() {
        return Err(format!(
            "Image dimensions are not powers of two ({width}x{height})"
        ));
    }
    let has_transparency = image.pixels().any(|p| p.2[3] < 255);
    let format = if has_transparency {
        hemtt_paa::PaXType::DXT5
    } else {
        hemtt_paa::PaXType::DXT1
    };
    let paa = hemtt_paa::Paa::from_dynamic(&image, format).map_err(|e| e.to_string())?;
    paa.write(&mut out_file).map_err(|e| e.to_string())?;

    // Remove uncompressed image from virtual FS (so not included in pbo)
    in_path.vfs().remove_file().expect("vfs err");
    Ok(())
}

struct ConvertError {
    file: String,
    reason: String,
}
impl Code for ConvertError {
    fn ident(&self) -> &'static str {
        "CONVERT"
    }
    fn message(&self) -> String {
        format!("Failed to convert file `{}`", self.file)
    }
    fn help(&self) -> Option<String> {
        Some(self.reason.clone())
    }
}
impl ConvertError {
    #[must_use]
    pub fn code(path: &WorkspacePath, reason: String) -> Arc<dyn Code> {
        Arc::new(Self {
            file: path.as_str().to_string(),
            reason,
        })
    }
}
