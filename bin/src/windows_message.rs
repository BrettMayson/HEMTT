use winapi::{
    shared::minwindef::DWORD,
    um::{
        wincon::{GetConsoleProcessList, GetConsoleWindow},
        winuser::{IDYES, MB_ICONQUESTION, MB_YESNO, MessageBoxW},
    },
};

pub fn check_no_terminal() {
    if is_console_created_for_this_program() {
        message();
        std::process::exit(1);
    }
}

fn is_console_created_for_this_program() -> bool {
    unsafe {
        if GetConsoleWindow().is_null() {
            return false;
        }

        // Get the number of processes attached to the console
        let mut process_list: [DWORD; 2] = [0; 2];
        let process_count = GetConsoleProcessList(
            process_list.as_mut_ptr(),
            DWORD::try_from(process_list.len()).unwrap_or(u32::MAX),
        );

        // If only one process is attached, the console was likely created for this program
        process_count == 1
    }
}

fn message() {
    let message = "HEMTT is a command-line tool intended to be used in a terminal. To use it, open a terminal (like in Visual Studio Code) and run it there.\n\nWould you like to open The HEMTT Book for more information?";
    let title = "HEMTT";

    // Convert strings to wide strings for the Windows API
    let message_wide: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title_wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();

    let response = unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };

    if response == IDYES
        && let Err(e) = webbrowser::open("https://hemtt.dev/")
    {
        eprintln!("Failed to open the HEMTT book: {e}");
    }
}
