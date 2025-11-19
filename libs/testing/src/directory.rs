use std::{path::PathBuf, sync::atomic::AtomicBool};

static GLOBAL_LOCK: AtomicBool = AtomicBool::new(false);

/// A temporary directory that is deleted when dropped.
///
/// Only one `TemporaryDirectory` can exist at a time, to avoid
/// issues with changing the current working directory.
///
/// Subsequent attempts to create a `TemporaryDirectory` while one
/// already exists will block until the existing one is dropped.
///
/// The current working directory is changed to the temporary directory, and
/// restored when the `TemporaryDirectory` is dropped.
#[derive(Debug)]
pub struct TemporaryDirectory {
    path: PathBuf,
    original: PathBuf,
}

impl TemporaryDirectory {
    #[must_use]
    /// Create a new `TemporaryDirectory`.
    ///
    /// # Panics
    /// - Panics if the directory cannot be created.
    /// - Panics if the current directory cannot be changed.
    /// - Panics if the global lock cannot be acquired within 5 minutes.
    pub fn new() -> Self {
        let start = std::time::Instant::now();
        loop {
            if GLOBAL_LOCK
                .compare_exchange(
                    false,
                    true,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                break;
            }
            assert!(
                (start.elapsed().as_secs() <= 300),
                "Could not acquire global lock for temporary directory"
            );
            std::thread::yield_now();
        }
        let random_id = rand::random::<u64>();
        let path = std::env::temp_dir().join(format!("hemtt_test_{random_id}"));
        std::fs::create_dir(&path).expect("Failed to create temporary directory");
        let original = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&path).expect("Failed to set current directory");
        Self { path, original }
    }

    #[must_use]
    /// Create a `TemporaryDirectory` by copying the contents of an existing directory.
    ///
    /// # Panics
    /// - Panics if the directory cannot be copied.
    /// - Panics if the current directory cannot be changed.
    /// - Panics if the global lock cannot be acquired within 5 minutes.
    pub fn copy(from: &std::path::Path) -> Self {
        let temp_dir = Self::new();
        fs_extra::dir::copy(
            from,
            &temp_dir.path,
            &fs_extra::dir::CopyOptions {
                content_only: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|_| {
            println!("from exists: {}", from.exists());
            println!("to exists: {}", temp_dir.path.exists());
            panic!(
                "Failed to copy directory from {} to {}",
                from.display(),
                temp_dir.path.display()
            )
        });
        std::env::set_current_dir(&temp_dir.path).expect("Failed to set current directory");
        temp_dir
    }
}

impl Default for TemporaryDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        GLOBAL_LOCK.store(false, std::sync::atomic::Ordering::SeqCst);
        std::env::set_current_dir(&self.original).expect("Failed to set current directory");
        std::fs::remove_dir_all(&self.path).expect("Failed to remove temporary directory");
    }
}

impl AsRef<std::path::Path> for TemporaryDirectory {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

impl std::ops::Deref for TemporaryDirectory {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::TemporaryDirectory;

    #[test]
    fn creation() {
        let temp_dir = TemporaryDirectory::new();
        assert!(temp_dir.as_ref().exists());
    }

    #[test]
    fn deletion() {
        let path_buf;
        {
            let temp_dir = TemporaryDirectory::new();
            path_buf = temp_dir.as_ref().to_path_buf();
            assert!(path_buf.exists());
        }
        assert!(!path_buf.exists());
    }

    #[test]
    fn lock() {
        let start = std::time::Instant::now();

        let handle1 = std::thread::spawn(move || {
            let _temp_dir = TemporaryDirectory::new();
            std::thread::sleep(std::time::Duration::from_millis(50));
        });
        let handle2 = std::thread::spawn(move || {
            let _temp_dir = TemporaryDirectory::new();
            std::thread::sleep(std::time::Duration::from_millis(50));
        });

        handle1.join().expect("Thread 1 panicked");
        handle2.join().expect("Thread 2 panicked");

        let elapsed = start.elapsed();
        // If threads ran serially (locked), total time should be ~100ms
        // If they ran in parallel (no lock), total time would be ~50ms
        assert!(
            elapsed.as_millis() >= 90,
            "Threads appear to have run in parallel ({}ms), lock may not be working",
            elapsed.as_millis()
        );
    }

    #[test]
    fn change_directory() {
        let original = std::env::current_dir().expect("Failed to get current directory");
        {
            let temp_dir = TemporaryDirectory::new();
            let current = std::env::current_dir().expect("Failed to get current directory");
            // Canonicalize paths to handle symlinks (e.g., /var -> /private/var on macOS)
            let current_canonical = current
                .canonicalize()
                .expect("Failed to canonicalize current directory");
            let temp_canonical = temp_dir
                .as_ref()
                .canonicalize()
                .expect("Failed to canonicalize temp directory");
            assert_eq!(current_canonical, temp_canonical);
            assert_ne!(current, original);
        }
        let current = std::env::current_dir().expect("Failed to get current directory");
        assert_eq!(current, original);
    }
}
