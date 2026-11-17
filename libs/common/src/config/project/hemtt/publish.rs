use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Visibility of a Steam Workshop item.
pub enum Visibility {
    Public,
    FriendsOnly,
    Private,
    Unlisted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
/// An additional Workshop gallery image or an existing `YouTube` video.
pub enum AdditionalPreview {
    Image { path: PathBuf },
    Youtube { video_id: String },
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for `hemtt publish`. Omitted metadata is left unchanged on updates.
pub struct PublishOptions {
    title: Option<String>,
    description: Option<PathBuf>,
    changelog: Option<PathBuf>,
    language: Option<String>,
    visibility: Option<Visibility>,
    tags: Option<Vec<String>>,
    preview: Option<PathBuf>,
    required_items: Option<Vec<String>>,
    additional_previews: Option<Vec<AdditionalPreview>>,
}

impl PublishOptions {
    /// Get the optional Workshop title override.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Get the description source.
    #[must_use]
    pub const fn description(&self) -> Option<&PathBuf> {
        self.description.as_ref()
    }

    /// Get the changelog source.
    #[must_use]
    pub const fn changelog(&self) -> Option<&PathBuf> {
        self.changelog.as_ref()
    }

    /// Get the Steam API language code for the title and description.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Get the requested Workshop visibility.
    #[must_use]
    pub const fn visibility(&self) -> Option<Visibility> {
        self.visibility
    }

    /// Get replacement tags. An omitted list preserves existing tags.
    #[must_use]
    pub fn tags(&self) -> Option<&[String]> {
        self.tags.as_deref()
    }

    /// Get the primary preview image.
    #[must_use]
    pub fn preview(&self) -> Option<&Path> {
        self.preview.as_deref()
    }

    /// Get replacement Required Items, expressed as decimal Workshop IDs.
    #[must_use]
    pub fn required_items(&self) -> Option<&[String]> {
        self.required_items.as_deref()
    }

    /// Get the replacement gallery. An empty list clears additional previews.
    #[must_use]
    pub fn additional_previews(&self) -> Option<&[AdditionalPreview]> {
        self.additional_previews.as_deref()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
/// Publish options in `.hemtt/project.toml` or the separate `.hemtt/publish.toml`.
pub struct PublishOptionsFile {
    title: Option<String>,
    description: Option<PathBuf>,
    changelog: Option<PathBuf>,
    language: Option<String>,
    visibility: Option<Visibility>,
    tags: Option<Vec<String>>,
    preview: Option<PathBuf>,
    required_items: Option<Vec<String>>,
    additional_previews: Option<Vec<AdditionalPreview>>,
}

impl PublishOptionsFile {
    /// Load one configuration source, resolving file paths against the project root.
    ///
    /// # Errors
    /// Returns an error for unreadable files, invalid TOML, or conflicting sources.
    pub fn load(self, project_path: &Path) -> Result<PublishOptions, Error> {
        let publish_path = project_path.with_file_name("publish.toml");
        let source = if publish_path.exists() {
            if self != Self::default() {
                return Err(Error::ConfigInvalid(
                    "Publish options must be configured in either `project.toml` or `publish.toml`, not both.".to_string(),
                ));
            }
            toml::from_str::<Self>(&fs_err::read_to_string(publish_path)?)?
        } else {
            self
        };
        let root = project_path
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."));
        Ok(source.resolve(root))
    }

    fn resolve(self, root: &Path) -> PublishOptions {
        PublishOptions {
            title: self.title,
            description: self.description.map(|path| root.join(path)),
            changelog: self.changelog.map(|path| root.join(path)),
            language: self.language,
            visibility: self.visibility,
            tags: self.tags,
            preview: self.preview.map(|path| root.join(path)),
            required_items: self.required_items,
            additional_previews: self.additional_previews.map(|previews| {
                previews
                    .into_iter()
                    .map(|preview| match preview {
                        AdditionalPreview::Image { path } => AdditionalPreview::Image {
                            path: root.join(path),
                        },
                        video @ AdditionalPreview::Youtube { .. } => video,
                    })
                    .collect()
            }),
        }
    }
}

#[cfg(test)]
mod tests;
