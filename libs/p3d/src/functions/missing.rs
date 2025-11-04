use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use hemtt_workspace::{Error, WorkspacePath};

use crate::P3D;

#[derive(Default)]
pub struct SearchCache {
    cache: RwLock<HashMap<String, Option<(u64, u64)>>>,
}

impl SearchCache {
    /// Create a new `SearchCache` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(128)),
        }
    }

    /// Check if the given file exists
    ///
    /// # Panics
    /// Panics if the cache is poisoned
    pub fn exists(&self, file_id: &str) -> Option<bool> {
        let cache = self.cache.read().expect("failed to lock cache");
        cache.get(file_id).copied().map(|path| path.is_some())
    }

    /// Insert a file into the cache
    ///
    /// # Panics
    /// Panics if the cache is poisoned
    pub fn insert(&self, file_id: String, metadata: Option<(u64, u64)>) {
        let mut cache = self.cache.write().expect("failed to lock cache");
        cache.insert(file_id, metadata);
    }

    /// Get the metadata for a file
    ///
    /// # Panics
    /// Panics if the cache is poisoned
    pub fn get_metadata(&self, file_id: &str) -> Option<(u64, u64)> {
        if file_id.is_empty() || file_id.starts_with('#') {
            return None;
        }
        let file_id = if file_id.starts_with('\\') {
            file_id.to_string()
        } else {
            format!("\\{file_id}")
        };
        let cache = self.cache.read().expect("failed to lock cache");
        cache.get(&file_id).copied().flatten()
    }
}

impl P3D {
    #[allow(clippy::too_many_lines)]
    /// Find missing textures and materials in the P3D
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    ///
    /// # Panics
    /// Panics if the modified time cannot be retrieved
    pub fn missing(
        &self,
        workspace: &WorkspacePath,
        cache: &SearchCache,
    ) -> Result<(Vec<String>, Vec<String>), Error> {
        let mut textures = HashSet::new();
        let mut materials = HashSet::new();
        for lod in &self.lods {
            for face in &lod.faces {
                textures.insert(face.texture.clone());
                materials.insert(face.material.clone());
            }
        }
        let mut missing_textures = Vec::new();
        for texture in textures {
            if texture.is_empty() || texture.starts_with('#') {
                continue;
            }
            let texture = if texture.starts_with('\\') {
                texture
            } else {
                format!("\\{texture}")
            };
            if let Some(exists) = cache.exists(&texture) {
                if !exists {
                    missing_textures.push(texture);
                }
            } else if let Some(located) = workspace.locate_with_pdrive(&texture)? {
                let metadata = located.path.metadata()?;
                cache.insert(
                    texture.clone(),
                    Some((
                        metadata
                            .modified
                            .expect("has modified")
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_else(|_| {
                                println!(
                                    "failed to epoch, {:?}",
                                    metadata.modified.expect("has modified")
                                );
                                std::time::Duration::from_secs(0)
                            })
                            .as_secs(),
                        metadata.len,
                    )),
                );
            } else {
                cache.insert(texture.clone(), None);
                missing_textures.push(texture);
            }
        }
        let mut missing_materials = Vec::new();
        for material in materials {
            if material.is_empty() || material.starts_with('#') {
                continue;
            }
            let material = if material.starts_with('\\') {
                material
            } else {
                format!("\\{material}")
            };
            if let Some(exists) = cache.exists(&material) {
                if !exists {
                    missing_materials.push(material);
                }
            } else if let Some(located) = workspace.locate_with_pdrive(&material)? {
                let metadata = located.path.metadata()?;
                cache.insert(
                    material.clone(),
                    Some((
                        metadata
                            .modified
                            .expect("has modified")
                            .duration_since(std::time::UNIX_EPOCH)
                            .expect("should be able to get duration since epoch")
                            .as_secs(),
                        metadata.len,
                    )),
                );
            } else {
                cache.insert(material.clone(), None);
                missing_materials.push(material);
            }
        }
        Ok((missing_textures, missing_materials))
    }
}
