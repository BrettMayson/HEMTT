use steamworks::PublishedFileId;

use crate::{
    Error,
    commands::{build::BuildArgs, release::ReleaseArgs},
    report::Report,
};

mod create;
mod native;
mod preflight;
mod text;
mod update;
mod workflow;

#[cfg(test)]
mod tests;

const APP_ID: steamworks::AppId = steamworks::AppId(107_410);

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Publish addon to Steam Workshop
///
/// `hemtt publish` will upload your mod to the Steam Workshop.
/// It builds your project using [`hemtt release`](./release.md) and uploads the build to the Steam Workshop, handling both initial creation and updates of existing items.
///
/// ## Getting Started
///
/// When you run `hemtt publish`, if you don't have a published item ID,
/// you will be prompted to create a new Steam Workshop item.
/// This will generate a `meta.cpp` file in the current directory with your published item ID.
/// Subsequent runs will update the existing item without prompting.
///
/// ## meta.cpp
///
/// The `meta.cpp` file stores your Steam Workshop published item ID.
/// It must be located in the root of your project (where you run `hemtt publish`).
///
/// ```
/// protocol = 1;
/// publishedid = 1234567890;
/// ```
///
/// ### Storing the ID
///
/// When you run `hemtt publish` and create a new item, HEMTT will automatically create or update your `meta.cpp` file with the `publishedid`.
/// This file should be committed to version control so the team can publish updates.
///
/// To manually set the published ID, add it to your `meta.cpp`:
/// ```cpp
/// protocol = 1;
/// publishedid = YOUR_WORKSHOP_ID_HERE;
/// ```
///
/// You can find your Workshop item ID in the URL on the Steam Community Hub: `https://steamcommunity.com/sharedfiles/filedetails/?id=YOUR_WORKSHOP_ID_HERE`
///
/// ## Workshop Configuration
///
/// Configure publishing in `.hemtt/publish.toml`,
/// or in `[hemtt.publish]` in `.hemtt/project.toml`.
/// Use one source only.
/// Paths in either source are relative to the project root.
/// The separate file has no table header:
///
/// ```toml
/// title = "Example Mod"
/// language = "english"
/// visibility = "private"
/// tags = ["Mod", "Weapon", "Editor Extension"]
/// description = "DESCRIPTION.md"
/// changelog = "CHANGELOG.md"
/// preview = "preview.png"
/// required_items = ["450814997", "463939057"]
/// additional_previews = [
///     { type = "image", path = "gallery/screenshot.png" },
///     { type = "image", path = "gallery/screenshot.jpg" },
///     { type = "image", path = "gallery/animation.gif" },
///     { type = "youtube", video_id = "dQw4w9WgXcQ" },
/// ]
/// ```
///
/// All settings are optional.
/// On updates, omitted settings preserve the corresponding Workshop metadata.
/// On creation, the title defaults to the project name and tags default to `["Mod"]`.
/// Configured titles and tags also apply to subsequent updates.
/// HEMTT's automatic Arma metadata is preserved.
///
/// - `title`: the Workshop item title, at most 128 UTF-8 bytes.
/// - `language`: the [Steam API language code](https://partner.steamgames.com/doc/store/localization/languages) for the title and description in this update.
///   Steam defaults to `english`.
/// - `visibility`: `public`, `friends_only`, `private`, or `unlisted`.
/// - `tags`: the exact Steam tag strings, passed through without label conversion.
///   Duplicate strings are removed, preserving their first occurrence.
/// - `description`: a UTF-8 file containing Workshop formatting or Markdown, converted to Steam `BBCode`.
///   The converted text must be shorter than 8,000 bytes.
/// - `changelog`: a UTF-8 file following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
///   The current project version must have an entry.
///   Its notes become the Steam Change Note; when omitted, HEMTT submits `Version <version>`.
/// - `preview`: a local primary Workshop picture.
/// - `required_items`: Workshop item IDs as nonzero decimal strings, such as `CBA_A3` and ACE3 in the example.
///   These are Workshop dependencies, not DLC IDs.
/// - `additional_previews`: the ordered gallery, separate from `preview`.
///   Images use `type = "image"` and `path`; `YouTube` videos use `type = "youtube"` and the existing video's 11-character `video_id`.
///   Local video upload is not supported.
///
/// Configured `tags`, `required_items`, and `additional_previews` are complete replacement lists.
/// Omit a list to preserve it; set `[]` to clear it.
/// Gallery order follows the list.
/// Each publish replaces the managed gallery, including changed local images, rather than appending duplicate entries.
///
/// Before building or connecting to Steam, HEMTT checks all local metadata sources and reports validation errors together.
/// Primary and additional images must be readable, nonempty PNG, JPEG, or GIF files strictly smaller than 1,000,000 bytes,
/// with a matching filename extension.
/// HEMTT decodes images locally, including every GIF frame, and rejects invalid files without resizing or converting them.
/// `YouTube` availability and Workshop dependency existence require live testing.
///
/// The upload and Required Items updates are separate Steam operations.
/// HEMTT waits for their results and reports failures, including dependency failures after a successful content upload.
/// Such an error does not roll back the upload.
///
/// ## Build Configuration
///
/// `hemtt publish` uses the same build configuration as [`hemtt release`](./release.md).
/// All signing and archiving options apply to the released build before upload.
///
/// ## Dry Run
///
/// Use `--dry-run` to preview what would be submitted to the Steam Workshop without connecting to Steam or uploading anything.
/// This builds the release and prints the complete metadata, image formats, dimensions, sizes, video IDs, dependencies, Change Note, and content that would be sent.
///
/// ```bash
/// hemtt publish --dry-run
/// ```
///
pub struct Command {
    #[clap(flatten)]
    pub build: BuildArgs,

    #[clap(flatten)]
    pub release: ReleaseArgs,

    #[arg(long = "dry-run")]
    /// Preview the update that would be submitted to Steam without uploading
    ///
    /// Builds the release and prints all configured metadata, media, dependencies, Change Note, and content that would be sent to the Steam Workshop, without connecting to Steam or modifying the Workshop item.
    pub dry_run: bool,

    #[clap(flatten)]
    pub global: crate::GlobalArgs,
}

/// Execute the publish command after a complete local preflight.
///
/// # Errors
/// Returns build, configuration, or Steam Workshop errors.
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let meta_path = std::env::current_dir()?.join("meta.cpp");
    let id = if meta_path.exists() {
        Some(get_id()?)
    } else {
        None
    };
    let (mut report, prepared) = update::prepare(cmd, id)?;
    let Some(mut prepared) = prepared else {
        return Ok(report);
    };
    if cmd.dry_run {
        report.merge(prepared.finish_release(false)?);
        if report.failed() {
            return Ok(report);
        }
        prepared.print();
        return Ok(report);
    }
    let steam = native::SteamWorkshop::connect()?;
    let assigned_id = id.is_none();
    let (id, create) = if let Some(id) = id {
        (id, false)
    } else {
        prompt_create_item(&steam)?
    };
    report.merge(prepared.finish_release(assigned_id)?);
    if report.failed() {
        return Ok(report);
    }
    workflow::publish(
        &steam,
        id,
        &prepared.content_path,
        &prepared.metadata,
        create,
    )?;
    println!(
        "Updated on Steam Workshop: https://steamcommunity.com/sharedfiles/filedetails/?id={}",
        id.0
    );
    Ok(report)
}

/// Get a nonzero published file ID from meta.cpp.
///
/// # Errors
/// Returns an error for missing, malformed, zero, or overflowing IDs.
///
/// # Panics
/// Panics if the hardcoded regular expression cannot be compiled.
pub fn get_id() -> Result<PublishedFileId, Error> {
    let path = std::env::current_dir()?.join("meta.cpp");
    if !path.exists() {
        return Err(Error::PublishedIdNotFound);
    }
    let content = fs_err::read_to_string(path)?;
    let regex =
        regex::Regex::new(r"(?m)^\s*publishedid\s*=\s*(\d+)\s*;").expect("valid meta regex");
    let id = regex
        .captures(&content)
        .and_then(|capture| capture[1].parse::<u64>().ok())
        .filter(|id| *id > 0)
        .ok_or(Error::PublishedIdNotFound)?;
    Ok(PublishedFileId(id))
}

fn prompt_create_item(steam: &native::SteamWorkshop) -> Result<(PublishedFileId, bool), Error> {
    match dialoguer::Select::new()
        .with_prompt("Workshop item not found")
        .default(0)
        .items([
            "Create a new Workshop item",
            "Input existing Workshop ID",
            "Cancel",
        ])
        .interact()?
    {
        0 => {
            let id = steam.create_item()?;
            create::store_id(id.0)?;
            info!(
                "Created Workshop item {} and stored its ID in meta.cpp",
                id.0
            );
            Ok((id, true))
        }
        1 => {
            let id = dialoguer::Input::<u64>::new()
                .with_prompt("Enter existing Workshop ID")
                .validate_with(|value: &u64| {
                    if *value > 0 {
                        Ok(())
                    } else {
                        Err("Workshop ID must be nonzero")
                    }
                })
                .interact()?;
            create::store_id(id)?;
            Ok((PublishedFileId(id), false))
        }
        _ => Err(Error::Workshop("Publishing cancelled".to_string())),
    }
}

fn silence_steam_output<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let capture = hemtt_common::capture::OutputCapture::new();
    let result = f();
    capture.finish();
    result
}
