use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use hemtt_workspace::{Error, WorkspacePath};

use crate::P3D;

#[derive(Default)]
pub struct SearchCache {
    cache: RwLock<HashMap<String, bool>>,
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
        cache.get(file_id).copied()
    }

    /// Insert a file into the cache
    ///
    /// # Panics
    /// Panics if the cache is poisoned
    pub fn insert(&self, file_id: String, exists: bool) {
        let mut cache = self.cache.write().expect("failed to lock cache");
        cache.insert(file_id, exists);
    }
}

impl P3D {
    /// Find missing textures and materials in the P3D
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
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
            // let texture = texture.to_lowercase();
            if let Some(exists) = cache.exists(&texture) {
                if !exists {
                    missing_textures.push(texture);
                }
            } else if workspace.locate(&texture)?.is_none() {
                #[allow(clippy::case_sensitive_file_extension_comparisons)]
                // working on lowercase paths
                let (replaced, ext) = if texture.ends_with(".paa") {
                    (texture.replace(".paa", ".tga"), "tga")
                } else if texture.ends_with(".tga") {
                    (texture.replace(".tga", ".paa"), "paa")
                } else if texture.ends_with(".png") {
                    (texture.replace(".png", ".paa"), "paa")
                } else {
                    (texture.clone(), "")
                };
                if ext.is_empty() || workspace.locate(&replaced)?.is_none() {
                    cache.insert(texture.clone(), false);
                    missing_textures.push(texture);
                } else {
                    cache.insert(texture.clone(), true);
                }
            } else {
                cache.insert(texture.clone(), true);
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
            } else if workspace.locate(&material)?.is_none() {
                cache.insert(material.clone(), false);
                missing_materials.push(material);
            } else {
                cache.insert(material.clone(), true);
            }
        }
        Ok((missing_textures, missing_materials))
    }
}
