#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

// Include the filtered bindings (types, constants, enums) without extern "C" blocks
#[cfg(target_os = "windows")]
include!(concat!(env!("OUT_DIR"), "/bindings_no_externs.rs"));

#[cfg(target_os = "macos")]
include!(concat!(env!("OUT_DIR"), "/bindings_no_externs.rs"));

#[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
include!(concat!(env!("OUT_DIR"), "/bindings_no_externs.rs"));

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
include!(concat!(env!("OUT_DIR"), "/bindings_no_externs.rs"));

// Include the auto-generated wrapper functions that provide runtime dynamic loading
include!(concat!(env!("OUT_DIR"), "/steam_api_wrappers.rs"));

/// Get the Steam API library embedded as bytes
#[cfg(target_os = "windows")]
fn get_embedded_steam_library() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/steam_api64.dll"))
}

#[cfg(target_os = "macos")]
fn get_embedded_steam_library() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/libsteam_api.dylib"))
}

#[cfg(target_os = "linux")]
fn get_embedded_steam_library() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/libsteam_api.so"))
}

/// Platform-specific library names
#[cfg(target_os = "windows")]
const STEAM_LIBRARY_NAME: &str = "steam_api64.dll";

#[cfg(target_os = "macos")]
const STEAM_LIBRARY_NAME: &str = "libsteam_api.dylib";

#[cfg(target_os = "linux")]
const STEAM_LIBRARY_NAME: &str = "libsteam_api.so";

/// Compute a stable hash of the embedded library and library name
fn compute_library_hash() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    STEAM_LIBRARY_NAME.hash(&mut hasher);
    get_embedded_steam_library().hash(&mut hasher);
    hasher.finish()
}

/// Get the cache directory for extracted Steam libraries
fn get_steam_cache_dir() -> io::Result<PathBuf> {
    let cache_dir = if cfg!(target_os = "windows") {
        // On Windows, use %LOCALAPPDATA%\hemtt\steam_cache
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(appdata).join("hemtt").join("steam_cache")
        } else {
            std::env::temp_dir().join("hemtt_steam_cache")
        }
    } else if cfg!(target_os = "macos") {
        // On macOS, use ~/Library/Caches/hemtt/steam
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Caches")
                .join("hemtt")
                .join("steam")
        } else {
            std::env::temp_dir().join("hemtt_steam_cache")
        }
    } else {
        // On Linux, use ~/.cache/hemtt/steam or /tmp/hemtt_steam_cache
        if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            PathBuf::from(xdg_cache).join("hemtt").join("steam")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".cache")
                .join("hemtt")
                .join("steam")
        } else {
            std::env::temp_dir().join("hemtt_steam_cache")
        }
    };

    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Extract the embedded Steam library to cache directory and return its path
fn extract_steam_library() -> io::Result<PathBuf> {
    let cache_dir = get_steam_cache_dir()?;
    let lib_hash = compute_library_hash();
    let version_dir = cache_dir.join(format!("v{}", lib_hash));

    // Create version-specific directory
    fs::create_dir_all(&version_dir)?;

    let lib_path = version_dir.join(STEAM_LIBRARY_NAME);

    // Only write if it doesn't exist (assumes same hash = same library)
    if !lib_path.exists() {
        let embedded_lib = get_embedded_steam_library();
        fs::write(&lib_path, embedded_lib)?;

        // Set appropriate permissions on Unix
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&lib_path, Permissions::from_mode(0o755))?;
        }
    }

    Ok(lib_path)
}

/// Try to find Steam library in system paths first
#[cfg(target_os = "linux")]
fn find_system_steam_library() -> Option<PathBuf> {
    // Common Steam installation paths on Linux
    let possible_paths = [
        // Steam runtime paths (Flatpak / modern Steam)
        std::path::Path::new("/var/home")
            .join(std::env::var("USER").unwrap_or_default())
            .join(".local/share/Steam/steamrt64/libsteam_api.so"),
        // Regular Steam paths
        std::path::Path::new("/var/home")
            .join(std::env::var("USER").unwrap_or_default())
            .join(".local/share/Steam/linux64/libsteam_api.so"),
        // Proton Steam paths
        std::path::Path::new("/var/home")
            .join(std::env::var("USER").unwrap_or_default())
            .join(
                ".local/share/Steam/compatibilitytools.d/GE-Proton9-27/files/lib64/libsteam_api.so",
            ),
        // SDK installation paths
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".steam/sdk64/libsteam_api.so"))
            .unwrap_or_default(),
    ];

    for path in &possible_paths {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn find_system_steam_library() -> Option<PathBuf> {
    None
}

/// Load the Steam API library dynamically
/// This must be called before any steamworks functions are used
pub fn load_steam_library() -> Result<(), Box<dyn std::error::Error>> {
    // Use OnceLock to ensure this only runs once
    static LOADED: OnceLock<Result<(), String>> = OnceLock::new();

    let result = LOADED.get_or_init(|| {
        // Try to use system-installed Steam library first (if available)
        // But fall back to embedded library if initialization fails

        if let Some(system_path) = find_system_steam_library() {
            // Try system library first
            #[cfg(target_os = "windows")]
            {
                unsafe {
                    if let Ok(lib) = libloading::Library::new(&system_path) {
                        if let Ok(()) = initialize_steam_api(&lib) {
                            let _leaked = Box::leak(Box::new(lib));
                            return Ok(());
                        }
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                unsafe {
                    match libloading::os::unix::Library::open(
                        Some(&system_path),
                        libloading::os::unix::RTLD_GLOBAL | libloading::os::unix::RTLD_LAZY,
                    ) {
                        Ok(lib) => match initialize_steam_api(&lib) {
                            Ok(()) => {
                                let _leaked = Box::leak(Box::new(lib));
                                return Ok(());
                            }
                            Err(_) => {}
                        },
                        Err(_) => {}
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                unsafe {
                    match libloading::os::unix::Library::open(
                        Some(&system_path),
                        libloading::os::unix::RTLD_GLOBAL | libloading::os::unix::RTLD_LAZY,
                    ) {
                        Ok(lib) => match initialize_steam_api(&lib) {
                            Ok(()) => {
                                let _leaked = Box::leak(Box::new(lib));
                                return Ok(());
                            }
                            Err(_) => {}
                        },
                        Err(_) => {}
                    }
                }
            }
        }

        // Fall back to embedded library
        let embedded_path = extract_steam_library().map_err(|e| e.to_string())?;

        #[cfg(target_os = "windows")]
        {
            unsafe {
                let lib = libloading::Library::new(&embedded_path).map_err(|e| e.to_string())?;
                initialize_steam_api(&lib).map_err(|e| e.to_string())?;
                let _leaked = Box::leak(Box::new(lib));
            }
        }

        #[cfg(target_os = "linux")]
        {
            unsafe {
                let lib = libloading::os::unix::Library::open(
                    Some(&embedded_path),
                    libloading::os::unix::RTLD_GLOBAL | libloading::os::unix::RTLD_LAZY,
                )
                .map_err(|e| e.to_string())?;
                initialize_steam_api(&lib).map_err(|e| e.to_string())?;
                let _leaked = Box::leak(Box::new(lib));
            }
        }

        #[cfg(target_os = "macos")]
        {
            unsafe {
                let lib = libloading::os::unix::Library::open(
                    Some(&embedded_path),
                    libloading::os::unix::RTLD_GLOBAL | libloading::os::unix::RTLD_LAZY,
                )
                .map_err(|e| e.to_string())?;
                initialize_steam_api(&lib).map_err(|e| e.to_string())?;
                let _leaked = Box::leak(Box::new(lib));
            }
        }

        Ok(())
    });

    result.clone().map_err(|e| e.into())
}

/// Cleanup function to remove old cached versions
/// This can be called periodically to clean up old extracted libraries
pub fn cleanup_old_versions() -> io::Result<()> {
    let cache_dir = get_steam_cache_dir()?;

    if !cache_dir.exists() {
        return Ok(());
    }

    let current_hash = compute_library_hash();
    let current_version = format!("v{}", current_hash);

    // Iterate through cache directory and remove old versions
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let dir_name = entry.file_name();
                    let dir_name_str = dir_name.to_string_lossy();

                    // Only remove version directories, not current version
                    if dir_name_str.starts_with('v') && dir_name_str != current_version {
                        let _ = fs::remove_dir_all(entry.path());
                    }
                }
            }
        }
    }

    Ok(())
}
