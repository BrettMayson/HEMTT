use winapi::{
    shared::minwindef::DWORD,
    um::{
        wincon::{GetConsoleProcessList, GetConsoleWindow},
        winuser::{MessageBoxW, IDYES, MB_ICONQUESTION, MB_YESNO},
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
        // Check if there is a console window
        if GetConsoleWindow().is_null() {
            // No console window is attached
            return false;
        }

        // Get the number of processes attached to the console
        let mut process_list: [DWORD; 2] = [0; 2];
        let process_count =
            GetConsoleProcessList(process_list.as_mut_ptr(), process_list.len() as DWORD);

        // If only one process is attached, the console was likely created for this program
        process_count == 1
    }
}

fn message() {
    let message = "This is a command-line tool. To use it, open a terminal and run it there.\n\nWould you like to open the documentation?";
    let title = "CLI Tool - Information";

    // Convert strings to wide strings for the Windows API
    let message_wide: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title_wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();

    // Show the message box with Yes/No buttons
    let response = unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };

    if response == IDYES {
        if let Err(e) = webbrowser::open("https://hemtt.dev/") {
            eprintln!("Failed to open the HEMTT book: {e}");
        }
    }
}
