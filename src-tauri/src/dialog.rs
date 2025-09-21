use winapi::um::winuser::{MessageBoxW, MB_OK, MB_ICONINFORMATION, MB_ICONWARNING};
use std::os::raw::c_int;
use std::ptr;

pub fn show_webview2_dialog() -> c_int {
    let message = "Better Steam AutoCracker requires Microsoft WebView2.\n\n\
                   It appears that WebView2 is not installed on your system.\n\n
                   Please download and install it from the official Microsoft website.";
    let title = "WebView2 Required";

    let message_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            message_w.as_ptr(),
            title_w.as_ptr(),
            MB_OK | MB_ICONWARNING,
        )
    }
}

pub fn show_foss_dialog() -> c_int {
    let message = "Better Steam AutoCracker is a FOSS software.\n\n\
                   If you have paid for this software, please demand a chargeback immediately.\n\n\
                   This program is provided under an MIT license, and you can find the source code on the GitHub repository.";
    let title = "Important";

    let message_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            message_w.as_ptr(),
            title_w.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        )
    }
}