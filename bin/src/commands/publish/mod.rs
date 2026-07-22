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
/// Publish addon to Steam Workshop. If not already published, offers to create a new item.
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
    // Load the Steam API library dynamically before any Steam functionality is used.
    // This extracts the embedded library and loads it with RTLD_GLOBAL (on Unix)
    // or adjusts DLL search paths (on Windows), ensuring Steamworks can find it.
    if let Err(e) = steamworks_sys::load_steam_library() {
        error!("Warning: Failed to preload Steam API library: {e}");
        std::process::exit(1);
    }

    let client = silence_steam_output(|| match steamworks::Client::init_app(APP_ID) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to initialize Steam client. Is Steam running?");
            error!("Error: {e}");
            std::process::exit(1);
        }
    });
    info!("Steam client initialized successfully.");
    let ugc = client.ugc();
    // create a channel to communicate with the upcoming callback thread
    // this is technically not *needed* but it is cleaner in order to properly exit the thread
    let (tx, rx) = std::sync::mpsc::channel();
    // create a thread for callbacks
    // if you have an active loop (like in a game), you can skip this and just run the callbacks on update
    let callback_thread = std::thread::spawn(move || {
        loop {
            // run callbacks
            client.run_callbacks();
            std::thread::sleep(std::time::Duration::from_millis(100));

            // check if the channel is closed or if there is a message
            // end the thread if either is true
            match rx.try_recv() {
                Ok(()) | Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }
        }
    });

    // Check if already published
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

struct FdRestore {
    stdout: libc::c_int,
    stderr: libc::c_int,
}

#[cfg(unix)]
impl Drop for FdRestore {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.stdout, libc::STDOUT_FILENO);
            libc::dup2(self.stderr, libc::STDERR_FILENO);
            libc::close(self.stdout);
            libc::close(self.stderr);
        }
    }
}

#[cfg(windows)]
impl Drop for FdRestore {
    fn drop(&mut self) {
        unsafe {
            libc::_dup2(self.stdout, libc::_fileno(libc::stdout));
            libc::_dup2(self.stderr, libc::_fileno(libc::stderr));
            libc::_close(self.stdout);
            libc::_close(self.stderr);
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

        let _restore = FdRestore { stdout, stderr };

        f()
    }
}

#[cfg(windows)]
fn silence_steam_output<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let stdout_fd = libc::_fileno(libc::stdout);
        let stderr_fd = libc::_fileno(libc::stderr);

        let stdout = libc::_dup(stdout_fd);
        let stderr = libc::_dup(stderr_fd);

        let devnull = libc::_open(c"NUL".as_ptr().cast(), libc::_O_WRONLY);

        libc::_dup2(devnull, stdout_fd);
        libc::_dup2(devnull, stderr_fd);
        libc::_close(devnull);

        let _restore = FdRestore { stdout, stderr };

        f()
    }
}
