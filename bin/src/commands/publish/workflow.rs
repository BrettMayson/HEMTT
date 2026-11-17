use std::{collections::BTreeSet, path::Path};

use steamworks::PublishedFileId;

use crate::Error;

use super::preflight::Metadata;

#[derive(Debug, Default)]
pub(super) struct RemoteState {
    pub required_items: Vec<u64>,
    pub additional_previews: u32,
}

pub(super) struct ItemUpdate<'a> {
    pub metadata: &'a Metadata,
    pub content: &'a Path,
    pub create: bool,
    pub remove_previews: u32,
}

/// The Steam boundary; workflow tests exercise sequencing without connecting to Steam.
pub(super) trait Workshop {
    fn inspect(
        &self,
        id: PublishedFileId,
        children: bool,
        previews: bool,
    ) -> Result<RemoteState, Error>;
    fn submit(&self, id: PublishedFileId, update: &ItemUpdate<'_>) -> Result<(), Error>;
    fn dependency(&self, id: PublishedFileId, child: u64, add: bool) -> Result<(), Error>;
}

/// Apply configured metadata and replace only explicitly managed remote lists.
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
