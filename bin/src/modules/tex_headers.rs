use std::sync::atomic::AtomicU16;

use hemtt_paa::TextureHeader;
use hemtt_workspace::addons::Addon;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

use crate::{modules::Module, progress::progress_bar};

#[derive(Debug, Default)]
pub struct TexHeaders;

impl Module for TexHeaders {
    fn name(&self) -> &'static str {
        "tex_headers"
    }

    fn pre_build(
        &self,
        ctx: &crate::context::Context,
    ) -> Result<crate::report::Report, crate::Error> {
        let addons = ctx
            .addons()
            .iter()
            .filter(|addon| {
                !ctx.workspace_path()
                    .join(addon.folder())
                    .expect("addon folder")
                    .read_dir()
                    .expect("read addon folder")
                    .iter()
                    .any(|entry| entry.filename().eq_ignore_ascii_case("texheaders.bin"))
            })
            .collect::<Vec<_>>();
        if addons.is_empty() {
            return Ok(crate::report::Report::new());
        }
        let progress = progress_bar(addons.len() as u64).with_message("Generating Texture Headers");
        let created = AtomicU16::new(0);
        let failed = AtomicU16::new(0);
        addons.par_iter().for_each(|addon| {
            let res = generate_texture_headers(ctx, addon);
            if let Err(e) = res {
                error!(
                    "Failed to generate texture headers for addon '{}': {}",
                    addon.folder(),
                    e
                );
                failed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                progress.inc(1);
                return;
            }
            if res.unwrap_or(false) {
                created.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            progress.inc(1);
        });
        progress.finish_and_clear();
        info!(
            "Generated {} texture headers",
            created.load(std::sync::atomic::Ordering::SeqCst)
        );
        if failed.load(std::sync::atomic::Ordering::SeqCst) > 0 {
            warn!(
                "{} texture headers failed to generate",
                failed.load(std::sync::atomic::Ordering::SeqCst)
            );
        }
        Ok(crate::report::Report::new())
    }
}

fn generate_texture_headers(
    ctx: &crate::context::Context,
    addon: &Addon,
) -> Result<bool, std::io::Error> {
    let mut textures = vec![];
    let root = addon.folder_pathbuf();
    let mut created = false;
    for entry in WalkDir::new(&root) {
        let entry = entry.expect("walk dir entry");
        let entry = entry.path();
        if entry.is_file() {
            let ext = entry
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_lowercase();
            if ["paa", "pax"].contains(&ext.as_str()) {
                textures.push(TextureHeader::from_file(&root, &entry.to_path_buf())?);
                created = true;
            }
        }
    }
    if textures.is_empty() {
        return Ok(false);
    }
    let headers = hemtt_paa::Headers::new(textures);
    let out_path = ctx
        .workspace_path()
        .join(addon.folder())
        .expect("addon folder")
        .join("texHeaders.bin")
        .expect("texHeaders.bin");
    debug!("Writing texture headers to {:?}", out_path);
    headers.write(&mut out_path.create_file().expect("create texHeaders.bin"))?;
    Ok(created)
}
