use std::mem::MaybeUninit;

use hemtt_common::config::Visibility;
use steamworks::PublishedFileId;
use steamworks_sys as sys;

use crate::Error;

use super::{CALL_TIMEOUT, SteamWorkshop, UPLOAD_TIMEOUT, checked, path_string, string, succeeded};
use crate::commands::publish::{
    preflight::GalleryPreview,
    workflow::{ItemUpdate, RemoteState, Workshop},
};

struct Query<'a> {
    steam: &'a SteamWorkshop,
    handle: sys::UGCQueryHandle_t,
}

impl Drop for Query<'_> {
    fn drop(&mut self) {
        // SAFETY: queries borrow the client, and the handle was checked on creation.
        unsafe {
            sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(self.steam.ugc, self.handle);
        }
    }
}

impl SteamWorkshop {
    pub(in crate::commands::publish) fn create_item(&self) -> Result<PublishedFileId, Error> {
        // SAFETY: the live client owns the UGC interface; no borrowed data is retained.
        let call = unsafe {
            sys::SteamAPI_ISteamUGC_CreateItem(
                self.ugc,
                super::super::APP_ID.0,
                sys::EWorkshopFileType::k_EWorkshopFileTypeCommunity,
            )
        };
        let result: sys::CreateItemResult_t = self.wait(
            call,
            sys::CreateItemResult_t_k_iCallback as i32,
            CALL_TIMEOUT,
            "Creating Workshop item",
        )?;
        succeeded(result.m_eResult, "Creating Workshop item")?;
        if result.m_nPublishedFileId == 0 {
            return Err(Error::Workshop(
                "Steam created an item without a valid ID".to_string(),
            ));
        }
        if result.m_bUserNeedsToAcceptWorkshopLegalAgreement {
            warn!(
                "Steam requires acceptance of the Workshop legal agreement before publishing can complete"
            );
        }
        Ok(PublishedFileId(result.m_nPublishedFileId))
    }
}

impl Workshop for SteamWorkshop {
    fn inspect(
        &self,
        id: PublishedFileId,
        children: bool,
        previews: bool,
    ) -> Result<RemoteState, Error> {
        let mut file_id = id.0;
        // SAFETY: Steam copies this one-element ID array during query creation.
        let handle = unsafe {
            sys::SteamAPI_ISteamUGC_CreateQueryUGCDetailsRequest(self.ugc, &raw mut file_id, 1)
        };
        if handle == sys::k_UGCQueryHandleInvalid {
            return Err(Error::Workshop(
                "Could not create the Workshop details query".to_string(),
            ));
        }
        let query = Query {
            steam: self,
            handle,
        };
        // SAFETY: the query guard and client keep these handles valid until the result is copied.
        unsafe {
            checked(
                sys::SteamAPI_ISteamUGC_SetReturnChildren(self.ugc, handle, children),
                "query child settings",
            )?;
            checked(
                sys::SteamAPI_ISteamUGC_SetReturnAdditionalPreviews(self.ugc, handle, previews),
                "query preview settings",
            )?;
            checked(
                sys::SteamAPI_ISteamUGC_SetAllowCachedResponse(self.ugc, handle, 0),
                "uncached Workshop query",
            )?;
        }
        let call = unsafe { sys::SteamAPI_ISteamUGC_SendQueryUGCRequest(self.ugc, handle) };
        let result: sys::SteamUGCQueryCompleted_t = self.wait(
            call,
            sys::SteamUGCQueryCompleted_t_k_iCallback as i32,
            CALL_TIMEOUT,
            "Querying Workshop item",
        )?;
        succeeded(result.m_eResult, "Querying Workshop item")?;
        if result.m_unNumResultsReturned != 1 {
            return Err(Error::Workshop(
                "Workshop item was not found or is inaccessible".to_string(),
            ));
        }
        let mut details = MaybeUninit::<sys::SteamUGCDetails_t>::uninit();
        checked(
            unsafe {
                sys::SteamAPI_ISteamUGC_GetQueryUGCResult(self.ugc, handle, 0, details.as_mut_ptr())
            },
            "Workshop item details",
        )?;
        // SAFETY: a successful GetQueryUGCResult initialized the full structure.
        let details = unsafe { details.assume_init() };
        succeeded(details.m_eResult, "Reading Workshop item")?;
        if details.m_nConsumerAppID != super::super::APP_ID.0 || details.m_nPublishedFileId != id.0
        {
            return Err(Error::Workshop(
                "The published ID does not identify the requested Arma 3 Workshop item".to_string(),
            ));
        }
        let mut required_items = Vec::new();
        if children && details.m_unNumChildren > 0 {
            required_items.resize(details.m_unNumChildren as usize, 0);
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_GetQueryUGCChildren(
                        self.ugc,
                        handle,
                        0,
                        required_items.as_mut_ptr(),
                        details.m_unNumChildren,
                    )
                },
                "Workshop Required Items",
            )?;
        }
        let additional_previews = if previews {
            unsafe { sys::SteamAPI_ISteamUGC_GetQueryUGCNumAdditionalPreviews(self.ugc, handle, 0) }
        } else {
            0
        };
        drop(query);
        Ok(RemoteState {
            required_items,
            additional_previews,
        })
    }

    fn submit(&self, id: PublishedFileId, update: &ItemUpdate<'_>) -> Result<(), Error> {
        let metadata = update.metadata;
        let content = path_string(update.content)?;
        // SAFETY: all setter strings live through each synchronous SDK call.
        // The SDK copies their contents into this pending update before returning.
        let handle = unsafe {
            sys::SteamAPI_ISteamUGC_StartItemUpdate(self.ugc, super::super::APP_ID.0, id.0)
        };
        if handle == sys::k_UGCUpdateHandleInvalid {
            return Err(Error::Workshop(
                "Could not start Workshop item update".to_string(),
            ));
        }
        checked(
            unsafe { sys::SteamAPI_ISteamUGC_SetItemContent(self.ugc, handle, content.as_ptr()) },
            "mod content",
        )?;
        if let Some(language) = &metadata.language {
            let language = string(language)?;
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_SetItemUpdateLanguage(
                        self.ugc,
                        handle,
                        language.as_ptr(),
                    )
                },
                "update language",
            )?;
        }
        if let Some(title) = metadata.effective_title(update.create) {
            let title = string(title)?;
            checked(
                unsafe { sys::SteamAPI_ISteamUGC_SetItemTitle(self.ugc, handle, title.as_ptr()) },
                "Workshop title",
            )?;
        }
        if let Some(description) = &metadata.description {
            let description = string(description)?;
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_SetItemDescription(
                        self.ugc,
                        handle,
                        description.as_ptr(),
                    )
                },
                "Workshop description",
            )?;
        }
        if let Some(visibility) = metadata.visibility {
            let visibility = match visibility {
                Visibility::Public => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityPublic,
                Visibility::FriendsOnly => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityFriendsOnly,
                Visibility::Private => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityPrivate,
                Visibility::Unlisted => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityUnlisted,
            };
            checked(
                unsafe { sys::SteamAPI_ISteamUGC_SetItemVisibility(self.ugc, handle, visibility) },
                "Workshop visibility",
            )?;
        }
        let default_tags = vec!["Mod".to_string()];
        if let Some(tags) = metadata
            .tags
            .as_deref()
            .or_else(|| update.create.then_some(default_tags.as_slice()))
        {
            let tags: Vec<_> = tags
                .iter()
                .map(|tag| string(tag))
                .collect::<Result<_, _>>()?;
            let mut pointers: Vec<_> = tags.iter().map(|tag| tag.as_ptr()).collect();
            let raw = sys::SteamParamStringArray_t {
                m_ppStrings: pointers.as_mut_ptr(),
                m_nNumStrings: i32::try_from(pointers.len())
                    .map_err(|error| Error::Workshop(error.to_string()))?,
            };
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_SetItemTags(self.ugc, handle, &raw const raw, true)
                },
                "Workshop tags",
            )?;
        }
        if update.create {
            // Preserve upstream's automatic Arma metadata, based on the project name.
            let platform = string("bis_platform")?;
            let platform_value = string("-")?;
            let name = string("bis_displayName")?;
            let project_name = string(&metadata.default_title)?;
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_AddItemKeyValueTag(
                        self.ugc,
                        handle,
                        platform.as_ptr(),
                        platform_value.as_ptr(),
                    )
                },
                "automatic Arma platform metadata",
            )?;
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_AddItemKeyValueTag(
                        self.ugc,
                        handle,
                        name.as_ptr(),
                        project_name.as_ptr(),
                    )
                },
                "automatic Arma display name",
            )?;
        }
        if let Some(preview) = &metadata.preview {
            let preview = path_string(&preview.path)?;
            checked(
                unsafe {
                    sys::SteamAPI_ISteamUGC_SetItemPreview(self.ugc, handle, preview.as_ptr())
                },
                "primary preview image",
            )?;
        }
        if let Some(previews) = &metadata.additional_previews {
            // Delete by descending index so removals cannot shift a later target.
            // Rebuilding this authoritative list also replaces changed image files at the same path, preserves order, and never appends duplicates.
            for index in (0..update.remove_previews).rev() {
                checked(
                    unsafe { sys::SteamAPI_ISteamUGC_RemoveItemPreview(self.ugc, handle, index) },
                    "removing an additional preview",
                )?;
            }
            for preview in previews {
                match preview {
                    GalleryPreview::Image(image) => {
                        let path = path_string(&image.path)?;
                        checked(
                            unsafe {
                                sys::SteamAPI_ISteamUGC_AddItemPreviewFile(
                                    self.ugc,
                                    handle,
                                    path.as_ptr(),
                                    sys::EItemPreviewType::k_EItemPreviewType_Image,
                                )
                            },
                            "adding a gallery image",
                        )?;
                    }
                    GalleryPreview::Youtube(video_id) => {
                        let video_id = string(video_id)?;
                        checked(
                            unsafe {
                                sys::SteamAPI_ISteamUGC_AddItemPreviewVideo(
                                    self.ugc,
                                    handle,
                                    video_id.as_ptr(),
                                )
                            },
                            "adding a YouTube preview",
                        )?;
                    }
                }
            }
        }
        let change_note = string(&metadata.change_note)?;
        let call = unsafe {
            sys::SteamAPI_ISteamUGC_SubmitItemUpdate(self.ugc, handle, change_note.as_ptr())
        };
        let result: sys::SubmitItemUpdateResult_t = self.wait(
            call,
            sys::SubmitItemUpdateResult_t_k_iCallback as i32,
            UPLOAD_TIMEOUT,
            "Uploading Workshop item",
        )?;
        succeeded(result.m_eResult, "Uploading Workshop item")?;
        if result.m_nPublishedFileId != id.0 {
            return Err(Error::Workshop(
                "Steam returned an unexpected uploaded item ID".to_string(),
            ));
        }
        if result.m_bUserNeedsToAcceptWorkshopLegalAgreement {
            return Err(Error::Workshop("Steam requires acceptance of the Workshop legal agreement; publication is not complete".to_string()));
        }
        Ok(())
    }

    fn dependency(&self, id: PublishedFileId, child: u64, add: bool) -> Result<(), Error> {
        if add {
            let call = unsafe { sys::SteamAPI_ISteamUGC_AddDependency(self.ugc, id.0, child) };
            let result: sys::AddUGCDependencyResult_t = self.wait(
                call,
                sys::AddUGCDependencyResult_t_k_iCallback as i32,
                CALL_TIMEOUT,
                "Adding Required Item",
            )?;
            succeeded(result.m_eResult, "Adding Required Item")
        } else {
            let call = unsafe { sys::SteamAPI_ISteamUGC_RemoveDependency(self.ugc, id.0, child) };
            let result: sys::RemoveUGCDependencyResult_t = self.wait(
                call,
                sys::RemoveUGCDependencyResult_t_k_iCallback as i32,
                CALL_TIMEOUT,
                "Removing Required Item",
            )?;
            succeeded(result.m_eResult, "Removing Required Item")
        }
    }
}
