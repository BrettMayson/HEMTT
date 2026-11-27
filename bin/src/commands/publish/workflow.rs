//! The order of Steam operations for one publish, independent of the Steam SDK
//!
//! 1. Inspect the existing item, only when a managed list needs its current contents
//! 2. Submit one item update: content, metadata and gallery
//! 3. Reconcile Required Items, which Steam only changes through separate calls

use std::{collections::BTreeSet, path::Path};

use steamworks::PublishedFileId;

use crate::Error;

use super::preflight::Metadata;

#[derive(Debug, Default)]
/// The parts of an existing Workshop item that a replacement list needs to know about
pub(super) struct RemoteState {
    /// Current Required Items, only queried when `required_items` is configured
    pub required_items: Vec<u64>,
    /// Number of additional previews currently on the item, images and videos,
    /// including any added by hand on the Workshop page.
    /// Only queried when `additional_previews` is configured.
    pub additional_previews: u32,
}

/// One `SubmitItemUpdate` call
pub(super) struct ItemUpdate<'a> {
    /// The validated metadata to apply
    pub metadata: &'a Metadata,
    /// The release folder to upload as the item content
    pub content: &'a Path,
    /// Whether the item was created by this publish, which applies the creation defaults
    pub create: bool,
    /// How many existing additional previews to remove before adding the configured gallery.
    /// This is every remote preview, Steam has no way to tell which ones HEMTT added.
    pub remove_previews: u32,
}

/// The Steam boundary; workflow tests exercise sequencing without connecting to Steam.
pub(super) trait Workshop {
    /// Read the item's current Required Items (`children`) and additional preview count (`previews`)
    fn inspect(
        &self,
        id: PublishedFileId,
        children: bool,
        previews: bool,
    ) -> Result<RemoteState, Error>;
    /// Upload the content and metadata, waiting for Steam to finish
    fn submit(&self, id: PublishedFileId, update: &ItemUpdate<'_>) -> Result<(), Error>;
    /// Add (`add = true`) or remove `child` from the item's Required Items
    fn dependency(&self, id: PublishedFileId, child: u64, add: bool) -> Result<(), Error>;
}

/// Apply configured metadata and replace only explicitly managed remote lists.
///
/// # Errors
/// Any Steam failure. A failure while updating Required Items happens after the content
/// was uploaded, and the upload is not rolled back.
pub(super) fn publish(
    steam: &impl Workshop,
    id: PublishedFileId,
    content: &Path,
    metadata: &Metadata,
    create: bool,
) -> Result<(), Error> {
    if metadata
        .required_items
        .as_ref()
        .is_some_and(|items| items.contains(&id.0))
    {
        return Err(Error::Workshop("An item cannot require itself".to_string()));
    }
    let children = metadata.required_items.is_some();
    let previews = metadata.additional_previews.is_some();
    let remote = if !create && (children || previews) {
        steam.inspect(id, children, previews)?
    } else {
        RemoteState::default()
    };
    steam.submit(
        id,
        &ItemUpdate {
            metadata,
            content,
            create,
            remove_previews: remote.additional_previews,
        },
    )?;
    if let Some(required) = &metadata.required_items {
        let existing: BTreeSet<_> = remote.required_items.into_iter().collect();
        let desired: BTreeSet<_> = required.iter().copied().collect();
        // Add replacements first so an addition failure does not remove existing dependencies.
        for &child in desired.difference(&existing) {
            steam.dependency(id, child, true).map_err(|error| {
                Error::Workshop(format!(
                    "Item upload succeeded, but adding Required Item {child} failed: {error}"
                ))
            })?;
        }
        for &child in existing.difference(&desired) {
            steam.dependency(id, child, false).map_err(|error| {
                Error::Workshop(format!(
                    "Item upload succeeded, but removing Required Item {child} failed: {error}"
                ))
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
