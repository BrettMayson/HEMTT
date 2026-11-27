//! Local validation of every Workshop metadata source, run before building or connecting to Steam
//!
//! Every problem is collected and reported at once, so a misconfigured project fails fast
//! instead of after a release build or a partial upload.

use std::{
    collections::HashSet,
    io::Cursor,
    path::{Path, PathBuf},
};

use hemtt_common::{
    config::{AdditionalPreview, PublishOptions, Visibility},
    version::Version,
};
use image::{AnimationDecoder, GenericImageView, ImageDecoder, ImageFormat};
use steamworks::PublishedFileId;

use crate::Error;

/// Largest accepted preview image, exclusive
///
/// A conservative reading of Steam's "under 1 MB" limit for preview images.
pub(super) const MAX_IMAGE_BYTES: usize = 1_000_000;
/// Longest accepted tag, in bytes
const MAX_TAG_BYTES: usize = 255;
/// Steam API language codes accepted for `language`
///
/// <https://partner.steamgames.com/doc/store/localization/languages>
const LANGUAGES: &[&str] = &[
    "arabic",
    "bulgarian",
    "schinese",
    "tchinese",
    "czech",
    "danish",
    "dutch",
    "english",
    "finnish",
    "french",
    "german",
    "greek",
    "hungarian",
    "indonesian",
    "italian",
    "japanese",
    "koreana",
    "malay",
    "norwegian",
    "polish",
    "portuguese",
    "brazilian",
    "romanian",
    "russian",
    "spanish",
    "latam",
    "swedish",
    "thai",
    "turkish",
    "ukrainian",
    "vietnamese",
];

#[derive(Debug, Clone)]
/// A local image that passed [`validate_image`]
pub(super) struct PreviewImage {
    /// Canonical, UTF-8 path, as Steam reads the file itself during the upload
    pub path: PathBuf,
    /// Detected format, which matches the filename extension
    pub format: ImageFormat,
    /// Width in pixels, shown by `--dry-run`
    pub width: u32,
    /// Height in pixels, shown by `--dry-run`
    pub height: u32,
    /// File size in bytes, below [`MAX_IMAGE_BYTES`]
    pub bytes: usize,
}

#[derive(Debug, Clone)]
/// A validated `additional_previews` entry
pub(super) enum GalleryPreview {
    /// A local image, uploaded on every publish
    Image(PreviewImage),
    /// An 11 character `YouTube` video ID, the video must already exist
    Youtube(String),
}

#[derive(Debug, Clone)]
/// Everything HEMTT will send to Steam, validated and converted
///
/// `None` means "leave the Workshop value unchanged" on updates.
/// On creation, [`Metadata::effective_title`] and the default `Mod` tag fill the gaps.
pub(super) struct Metadata {
    /// Configured title, applied on every publish
    pub title: Option<String>,
    /// The project name, used as the title on creation when none is configured,
    /// and always used for the `bis_displayName` key-value tag on creation
    pub default_title: String,
    /// Description converted to Workshop `BBCode`, see [`super::text::is_markdown`]
    pub description: Option<String>,
    /// Change Note, from the changelog entry or `Version <version>`
    pub change_note: String,
    /// Steam API language code that the title and description are written in
    pub language: Option<String>,
    /// Requested visibility
    pub visibility: Option<Visibility>,
    /// Replacement tags, deduplicated, keeping the first occurrence
    pub tags: Option<Vec<String>>,
    /// Primary preview image
    pub preview: Option<PreviewImage>,
    /// Replacement Required Items, deduplicated, never including the item itself
    pub required_items: Option<Vec<u64>>,
    /// Replacement gallery, in display order, `Some(vec![])` clears it
    pub additional_previews: Option<Vec<GalleryPreview>>,
}

impl Metadata {
    /// Validate every local source before building or initializing Steam.
    ///
    /// `id` is the existing published ID, `None` when a new item will be created.
    ///
    /// # Errors
    /// [`Error::Config`] listing every validation error found
    pub(super) fn prepare(
        options: &PublishOptions,
        project_name: &str,
        version: &Version,
        id: Option<PublishedFileId>,
    ) -> Result<Self, Error> {
        let mut errors = Vec::new();
        let title = options.title().map(str::to_string);
        if let Some(title) = title
            .as_deref()
            .or_else(|| id.is_none().then_some(project_name))
        {
            validate_text(
                "title",
                title,
                steamworks_sys::k_cchPublishedDocumentTitleMax as usize,
                &mut errors,
            );
            if title.trim().is_empty() {
                errors.push("title must not be empty".to_string());
            }
        }
        if id.is_none() && options.title().is_some() {
            validate_text(
                "project name for automatic Arma metadata",
                project_name,
                MAX_TAG_BYTES + 1,
                &mut errors,
            );
        }
        if let Some(language) = options.language()
            && !LANGUAGES.contains(&language)
        {
            errors.push(format!(
                "language '{language}' is not a supported Steam API language code"
            ));
        }
        let mut seen_tags = HashSet::new();
        let tags = options.tags().map(|tags| {
            tags.iter()
                .filter(|tag| seen_tags.insert((*tag).clone()))
                .cloned()
                .collect::<Vec<_>>()
        });
        if let Some(tags) = &tags {
            for tag in tags {
                if tag.is_empty()
                    || tag.len() > MAX_TAG_BYTES
                    || !tag
                        .bytes()
                        .all(|byte| (b' '..=b'~').contains(&byte) && byte != b',')
                {
                    errors.push(format!("tag '{tag}' must be 1..={MAX_TAG_BYTES} printable ASCII bytes without commas"));
                }
            }
        }
        let description = options.description().and_then(|path| {
            read_text(path, "description", &mut errors).and_then(|source| {
                let converted = if super::text::is_markdown(path) {
                    super::text::description(&source)
                } else {
                    Ok(source)
                };
                match converted {
                    Ok(text) => {
                        validate_text(
                            "description",
                            &text,
                            steamworks_sys::k_cchPublishedDocumentDescriptionMax as usize,
                            &mut errors,
                        );
                        Some(text)
                    }
                    Err(error) => {
                        errors.push(error);
                        None
                    }
                }
            })
        });
        let change_note = options.changelog().map_or_else(
            || Some(format!("Version {version}")),
            |path| {
                read_text(path, "changelog", &mut errors).and_then(|source| {
                    match super::text::change_note(&source, &version.to_string()) {
                        Ok(note) => Some(note),
                        Err(error) => {
                            errors.push(error);
                            None
                        }
                    }
                })
            },
        );
        if let Some(note) = &change_note {
            validate_text(
                "Change Note",
                note,
                steamworks_sys::k_cchPublishedDocumentChangeDescriptionMax as usize,
                &mut errors,
            );
        }
        let preview = options
            .preview()
            .and_then(|path| checked_image(path, &mut errors));
        let mut seen_ids = HashSet::new();
        let required_items = options.required_items().map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let parsed = item.parse::<u64>().ok().filter(|value| {
                        *value > 0 && item.bytes().all(|byte| byte.is_ascii_digit())
                    });
                    match parsed {
                        None => {
                            errors.push(format!(
                                "required_items contains invalid Workshop ID '{item}'"
                            ));
                            None
                        }
                        Some(value) if id.is_some_and(|id| id.0 == value) => {
                            errors.push(
                                "required_items must not include the item itself".to_string(),
                            );
                            None
                        }
                        Some(value) => seen_ids.insert(value).then_some(value),
                    }
                })
                .collect()
        });
        let mut seen_previews = HashSet::new();
        let additional_previews = options.additional_previews().map(|previews| previews.iter().filter_map(|preview| {
            let result = match preview {
                AdditionalPreview::Image { path } => checked_image(path, &mut errors).map(GalleryPreview::Image),
                AdditionalPreview::Youtube { video_id } => {
                    if video_id.len() != 11 || !video_id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
                        errors.push(format!("YouTube video_id '{video_id}' must contain exactly 11 ASCII letters, digits, '_' or '-'"));
                        None
                    } else { Some(GalleryPreview::Youtube(video_id.clone())) }
                }
            };
            if let Some(preview) = &result {
                let identity = match preview {
                    GalleryPreview::Image(image) => format!("image:{}", image.path.display()),
                    GalleryPreview::Youtube(id) => format!("youtube:{id}"),
                };
                if !seen_previews.insert(identity) {
                    errors.push("additional_previews contains a duplicate image path or video ID".to_string());
                }
            }
            result
        }).collect());
        if !errors.is_empty() {
            return Err(Error::Config(format!(
                "Workshop preflight failed:\n{}",
                errors
                    .iter()
                    .map(|error| format!("- {error}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
        Ok(Self {
            title,
            default_title: project_name.to_string(),
            description,
            change_note: change_note.unwrap_or_default(),
            language: options.language().map(str::to_string),
            visibility: options.visibility(),
            tags,
            preview,
            required_items,
            additional_previews,
        })
    }

    /// The title to send: the configured title, or the project name when creating an item
    ///
    /// `None` leaves the existing title unchanged.
    pub(super) fn effective_title(&self, create: bool) -> Option<&str> {
        self.title
            .as_deref()
            .or_else(|| create.then_some(self.default_title.as_str()))
    }
}

/// Check that `value` fits into Steam's fixed `buffer_bytes` buffer, including the NUL terminator
fn validate_text(field: &str, value: &str, buffer_bytes: usize, errors: &mut Vec<String>) {
    if value.len() >= buffer_bytes || value.contains('\0') {
        errors.push(format!(
            "{field} must be shorter than {buffer_bytes} UTF-8 bytes and contain no NUL characters"
        ));
    }
}

/// Read a UTF-8 source file, recording an error instead of returning it
fn read_text(path: &Path, field: &str, errors: &mut Vec<String>) -> Option<String> {
    match fs_err::read_to_string(path) {
        Ok(source) => Some(source),
        Err(error) => {
            errors.push(format!("{field}: {error}"));
            None
        }
    }
}

/// [`validate_image`], recording an error instead of returning it
fn checked_image(path: &Path, errors: &mut Vec<String>) -> Option<PreviewImage> {
    match validate_image(path) {
        Ok(image) => Some(image),
        Err(error) => {
            errors.push(format!("preview '{}': {error}", path.display()));
            None
        }
    }
}

/// Validate a preview image without modifying it
///
/// The file must be a non-empty PNG, JPEG or GIF smaller than [`MAX_IMAGE_BYTES`],
/// with a matching extension, that decodes fully (every frame for a GIF).
/// The returned path is canonical, since Steam reads the file itself.
///
/// # Errors
/// A description of the first problem found
pub(super) fn validate_image(path: &Path) -> Result<PreviewImage, String> {
    let metadata = fs_err::metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() >= MAX_IMAGE_BYTES as u64 {
        return Err(format!(
            "must be a non-empty regular image file smaller than {MAX_IMAGE_BYTES} bytes"
        ));
    }
    let bytes = fs_err::read(path).map_err(|error| error.to_string())?;
    if bytes.len() >= MAX_IMAGE_BYTES {
        return Err(format!("must be smaller than {MAX_IMAGE_BYTES} bytes"));
    }
    let format = image::guess_format(&bytes).map_err(|error| error.to_string())?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let matches = match format {
        ImageFormat::Png => extension == "png",
        ImageFormat::Jpeg => extension == "jpg" || extension == "jpeg",
        ImageFormat::Gif => extension == "gif",
        _ => false,
    };
    if !matches {
        return Err(
            "must contain PNG, JPEG, or GIF data matching its filename extension".to_string(),
        );
    }
    let image = image::ImageReader::with_format(Cursor::new(&bytes), format)
        .decode()
        .map_err(|error| error.to_string())?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err("image dimensions must be non-zero".to_string());
    }
    if format == ImageFormat::Gif {
        let mut decoder = image::codecs::gif::GifDecoder::new(Cursor::new(&bytes))
            .map_err(|error| error.to_string())?;
        decoder
            .set_limits(image::Limits::default())
            .map_err(|error| error.to_string())?;
        for frame in decoder.into_frames() {
            frame.map_err(|error| error.to_string())?;
        }
    }
    let path = fs_err::canonicalize(path).map_err(|error| error.to_string())?;
    if path.to_str().is_none() {
        return Err("image path must be representable as UTF-8 for Steam".to_string());
    }
    Ok(PreviewImage {
        path,
        format,
        width,
        height,
        bytes: bytes.len(),
    })
}

#[cfg(test)]
mod tests;
