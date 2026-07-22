use std::sync::mpsc::TryRecvError;

use steamworks::{PublishedFileId, UGC};

use crate::{
    Error,
    commands::{build::BuildArgs, release::ReleaseArgs},
    report::Report,
};

mod create;
mod update;

const APP_ID: steamworks::AppId = steamworks::AppId(107_410);

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Publish addon to Steam Workshop
///
/// `hemtt publish` will upload your mod to the Steam Workshop.
/// It builds your project using [`hemtt release`](./release.md) and uploads the build
/// to the Steam Workshop, handling both initial creation and updates of existing items.
///
/// ## Getting Started
///
/// When you run `hemtt publish`, if you don't have a published item ID, you will be prompted to create a new Steam Workshop item.
/// This will generate a `meta.cpp` file in the current directory with your published item ID.
/// Subsequent runs will update the existing item without prompting.
///
/// ## meta.cpp
///
/// The `meta.cpp` file stores your Steam Workshop published item ID. It must be located
/// in the root of your project (where you run `hemtt publish`).
///
/// ```
/// protocol = 1;
/// publishedid = 1234567890;
/// ```
///
/// ### Storing the ID
///
/// When you run `hemtt publish` and create a new item, HEMTT will
/// automatically create or update your `meta.cpp` file with the `publishedid`.
/// This file should be committed to version control so the team can publish updates.
///
/// To manually set the published ID, add it to your `meta.cpp`:
/// ```cpp
/// protocol = 1;
/// publishedid = YOUR_WORKSHOP_ID_HERE;
/// ```
///
/// You can find your Workshop item ID in the URL on the Steam Community Hub:
/// `https://steamcommunity.com/sharedfiles/filedetails/?id=YOUR_WORKSHOP_ID_HERE`
///
/// ## Configuration
///
/// `hemtt publish` uses the same build configuration as [`hemtt release`](./release.md).
/// All signing and archiving options apply to the released build before upload.
///
pub struct Command {
    #[clap(flatten)]
    pub build: BuildArgs,

    #[clap(flatten)]
    pub release: ReleaseArgs,

    #[clap(flatten)]
    pub global: crate::GlobalArgs,
}

/// Execute the publish command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// Panics if the Steam client cannot be initialized, or if the callback thread cannot be joined.
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    if let Err(e) = steamworks_sys::load_steam_library() {
        error!("Warning: Failed to preload Steam API library: {e}");
        std::process::exit(1);
    }

    let client = match silence_steam_output(|| steamworks::Client::init_app(APP_ID)) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to initialize Steam client. Is Steam running?");
            error!("Error: {e}");
            std::process::exit(1);
        }
    };
    info!("Steam client initialized successfully.");
    let ugc = client.ugc();
    let (tx, rx) = std::sync::mpsc::channel();
    let callback_thread = std::thread::spawn(move || {
        loop {
            client.run_callbacks();
            std::thread::sleep(std::time::Duration::from_millis(100));
            match rx.try_recv() {
                Ok(()) | Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }
        }
    });

    let has_id = get_id().is_ok();

    if !has_id {
        prompt_create_item(&ugc)?;
    }
    let result = update::execute(cmd, &ugc, !has_id)?;

    std::thread::sleep(std::time::Duration::from_secs(1));
    tx.send(())
        .expect("Failed to send message to callback thread");
    callback_thread
        .join()
        .expect("Failed to join callback thread");
    Ok(result)
}

/// Get the published file ID from meta.cpp
///
/// # Errors
/// [`Error`] if the file cannot be read or the ID cannot be found
///
/// # Panics
/// Panics if the regex cannot be compiled, which should never happen since the regex is hardcoded and valid.
pub fn get_id() -> Result<PublishedFileId, Error> {
    let mut meta = None;
    let meta_path = std::env::current_dir()?.join("meta.cpp");
    if meta_path.exists() {
        let content = fs_err::read_to_string(meta_path)?;
        let regex = regex::Regex::new(r"publishedid\s*=\s*(\d+);").expect("meta regex compiles");
        if let Some(id) = regex.captures(&content).map(|c| c[1].to_string()) {
            meta = Some(id);
        }
    }
    let Some(meta) = meta else {
        return Err(Error::PublishedIdNotFound);
    };
    let id = meta
        .parse::<u64>()
        .map_err(|_| Error::PublishedIdNotFound)?;
    Ok(PublishedFileId(id))
}

/// Prompt the user if they want to create a new Steam Workshop item
fn prompt_create_item(ugc: &UGC) -> Result<Report, Error> {
    match dialoguer::Select::new()
        .with_prompt("Workshop item not found")
        .default(0)
        .items([
            "Create a new Workshop item",
            "Input existing Workshop ID",
            "Cancel",
        ])
        .interact()
    {
        Ok(0) => create::execute(ugc),
        Ok(1) => {
            let id = dialoguer::Input::<u64>::new()
                .with_prompt("Enter existing Workshop ID")
                .interact()
                .expect("Failed to read input");
            create::store_id(id)?;
            Ok(Report::new())
        }
        _ => {
            error!("No Workshop item found. Please create one or add the ID to meta.cpp.");
            std::process::exit(1);
        }
    }
}

#[cfg(unix)]
struct FdRestore {
    stdout: libc::c_int,
    stderr: libc::c_int,
    devnull: libc::c_int,
}

#[cfg(unix)]
impl Drop for FdRestore {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.stdout, libc::STDOUT_FILENO);
            libc::dup2(self.stderr, libc::STDERR_FILENO);
            libc::close(self.stdout);
            libc::close(self.stderr);
            libc::close(self.devnull);
        }
    }
}

#[cfg(unix)]
fn silence_steam_output<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let stdout = libc::dup(libc::STDOUT_FILENO);
        let stderr = libc::dup(libc::STDERR_FILENO);

        let devnull = libc::open(c"/dev/null".as_ptr().cast(), libc::O_WRONLY);

        libc::dup2(devnull, libc::STDOUT_FILENO);
        libc::dup2(devnull, libc::STDERR_FILENO);
        libc::close(devnull);

        let _restore = FdRestore {
            stdout,
            stderr,
            devnull,
        };

        f()
    }
}

#[cfg(windows)]
struct StdHandleRestore {
    stdout: windows::Win32::Foundation::HANDLE,
    stderr: windows::Win32::Foundation::HANDLE,
    null: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Drop for StdHandleRestore {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Console::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle};

        unsafe {
            let _ = SetStdHandle(STD_OUTPUT_HANDLE, self.stdout);
            let _ = SetStdHandle(STD_ERROR_HANDLE, self.stderr);
            let _ = CloseHandle(self.null);
        }
    }
}

#[cfg(windows)]
fn silence_steam_output<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use std::ptr::null_mut;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
    };

    unsafe {
        let stdout = GetStdHandle(STD_OUTPUT_HANDLE).unwrap_or(HANDLE(null_mut()));
        let stderr = GetStdHandle(STD_ERROR_HANDLE).unwrap_or(HANDLE(null_mut()));

        let null = windows::Win32::Storage::FileSystem::CreateFileW(
            windows::core::w!("NUL"),
            windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE.0,
            windows::Win32::Storage::FileSystem::FILE_SHARE_READ
                | windows::Win32::Storage::FileSystem::FILE_SHARE_WRITE,
            None,
            windows::Win32::Storage::FileSystem::OPEN_EXISTING,
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            None,
        );

        let Ok(null) = null else {
            return f();
        };

        let _restore = StdHandleRestore {
            stdout,
            stderr,
            null,
        };

        let _ = SetStdHandle(STD_OUTPUT_HANDLE, null);
        let _ = SetStdHandle(STD_ERROR_HANDLE, null);

        f()
    }
}
