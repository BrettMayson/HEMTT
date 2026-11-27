//! Preparing a publish: validation, the release build, and `--dry-run` output

use std::path::PathBuf;

use hemtt_common::version::Version;
use steamworks::PublishedFileId;

use crate::{Error, report::Report, utils::bytes_to_human_readable};

use super::preflight::{GalleryPreview, Metadata, PreviewImage};

/// A validated publish, with the release built up to the `pre_release` stage
///
/// The release is finished by [`Prepared::finish_release`] once the published ID is known,
/// so a newly created ID is included in the uploaded `meta.cpp` and the archives.
pub(super) struct Prepared {
    /// The validated metadata to send
    pub metadata: Metadata,
    /// Canonical path of the release build folder that is uploaded
    pub content_path: PathBuf,
    /// Size of the content, shown by `--dry-run`
    content_size: u64,
    /// The project version, shown by `--dry-run`
    version: Version,
    /// The existing published ID, `None` if an item will be created
    id: Option<PublishedFileId>,
    /// The paused release build
    executor: crate::executor::Executor,
}

/// Validate local metadata first, then build exactly one release.
pub(super) fn prepare(
    cmd: &super::Command,
    id: Option<PublishedFileId>,
) -> Result<(Report, Option<Prepared>), Error> {
    // Preserve a previous release until all local metadata passes validation.
    let context =
        crate::context::Context::new(None, crate::context::PreservePrevious::Keep, false)?;
    let config = context.config();
    let version = config.version().get(context.workspace_path().vfs())?;
    let metadata = Metadata::prepare(config.hemtt().publish(), config.name(), &version, id)?;
    drop(context);
    let mut executor = crate::commands::release::executor(&cmd.release, &cmd.build)?;
    let report = executor.run_build()?;
    if report.failed() {
        return Ok((report, None));
    }
    let content_path = executor
        .ctx()
        .build_folder()
        .ok_or_else(|| Error::Config("Release build folder not found".to_string()))?;
    let content_path = fs_err::canonicalize(content_path)?;
    let content_size = dir_size(&content_path)?;
    Ok((
        report,
        Some(Prepared {
            metadata,
            content_path,
            content_size,
            version,
            id,
            executor,
        }),
    ))
}

impl Prepared {
    /// Run the remaining release stages: signing, archives, and so on
    ///
    /// `assigned_id` copies the `meta.cpp` written after creating an item into the content first.
    pub(super) fn finish_release(&mut self, assigned_id: bool) -> Result<Report, Error> {
        if assigned_id {
            // Files were copied before Steam assigned the first published ID.
            // Include it before Meta adds its timestamp and archives are produced.
            fs_err::copy(
                std::env::current_dir()?.join("meta.cpp"),
                self.content_path.join("meta.cpp"),
            )?;
        }
        let report = self.executor.run()?;
        self.content_size = dir_size(&self.content_path)?;
        Ok(report)
    }

    /// Print what would be sent to Steam, for `--dry-run`
    pub(super) fn print(&self) {
        let create = self.id.is_none();
        info!("Dry run, nothing was submitted to Steam");
        match self.id {
            Some(id) => println!("Workshop Item: #{} (existing item will be updated)", id.0),
            None => println!("Workshop Item: none found, a new item would be created"),
        }
        println!("Version: {}", self.version);
        println!(
            "Content: {} ({})",
            self.content_path.display(),
            bytes_to_human_readable(self.content_size)
        );
        println!(
            "Title: {}",
            self.metadata.effective_title(create).unwrap_or("unchanged")
        );
        println!(
            "Language: {}",
            self.metadata
                .language
                .as_deref()
                .unwrap_or("english (Steam default for edited text)")
        );
        println!(
            "Visibility: {}",
            match self.metadata.visibility {
                Some(hemtt_common::config::Visibility::Public) => "public",
                Some(hemtt_common::config::Visibility::FriendsOnly) => "friends_only",
                Some(hemtt_common::config::Visibility::Private) => "private",
                Some(hemtt_common::config::Visibility::Unlisted) => "unlisted",
                None if create => "Steam default",
                None => "unchanged",
            }
        );
        match &self.metadata.tags {
            Some(tags) => println!("Tags (replace): {tags:?}"),
            None if create => println!("Tags: Mod (creation default)"),
            None => println!("Tags: unchanged"),
        }
        match &self.metadata.preview {
            Some(image) => print_image("Preview", image),
            None => println!("Preview: unchanged"),
        }
        match &self.metadata.required_items {
            Some(items) => println!("Required Items (replace): {items:?}"),
            None => println!("Required Items: unchanged"),
        }
        match &self.metadata.additional_previews {
            Some(previews) => {
                println!("Additional previews (replace, in order):");
                if previews.is_empty() {
                    println!("  [] (clear gallery)");
                }
                for preview in previews {
                    match preview {
                        GalleryPreview::Image(image) => print_image("  Image", image),
                        GalleryPreview::Youtube(id) => println!("  YouTube video_id: {id}"),
                    }
                }
            }
            None => println!("Additional previews: unchanged"),
        }
        match &self.metadata.description {
            Some(text) => println!("\nDescription:\n{text}"),
            None => println!("\nDescription: unchanged"),
        }
        println!("\nChangelog:\n{}", self.metadata.change_note);
    }
}

/// Print one validated image line for `--dry-run`
fn print_image(label: &str, image: &PreviewImage) {
    println!(
        "{label}: {} ({:?}, {}x{}, {} bytes)",
        image.path.display(),
        image.format,
        image.width,
        image.height,
        image.bytes
    );
}

/// Total size of every file below `path`
fn dir_size(path: &std::path::Path) -> Result<u64, Error> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .try_fold(0, |total, entry| {
            let entry = entry?;
            Ok(total
                + if entry.file_type().is_file() {
                    entry.metadata()?.len()
                } else {
                    0
                })
        })
}
