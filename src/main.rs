use clipboard::Clipboard;
use tray::TrayIcon;
use url::Url;
use window::Window;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{DefWindowProcW, WM_CLIPBOARDUPDATE},
};

mod clipboard;
mod tray;
mod window;

const WINDOW_CLASS_NAME: &str = "CleanWeb";

pub type Win32Result<T> = Result<T, windows::core::Error>;

fn main() {
    let window = Window::new(WINDOW_CLASS_NAME, Some(message_handler)).unwrap();
    // Using this to let `Drop` clean up the tray icon
    let _tray_icon = TrayIcon::try_from(&window).unwrap();

    window.listen_to_clipboard().unwrap();
    window.run_message_loop();
}

/// A basic Windows message handler, deferring to the system-default WNDPROC
/// for messages we don't handle. Note that `WM_CLIPBOARDUPDATE` expects `0` when
/// the application handled the message and a non-zero result otherwise.
unsafe extern "system" fn message_handler(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CLIPBOARDUPDATE => {
            manage_clipboard(hwnd).map_or(
                LRESULT(1),
                |handled| if handled { LRESULT(0) } else { LRESULT(1) },
            )
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

/// The actual work to be done on the clipboard. Note that this _will_ be activated
/// by itself, so there needs to be no-op handling to break infinite recursion.
fn manage_clipboard(hwnd: HWND) -> Win32Result<bool> {
    if Clipboard::has_text() {
        let clipboard = Clipboard::from_handle(hwnd)?;
        let text = clipboard.get_contents()?;
        let updated_text = mangle_url(&text);

        if text == updated_text {
            Ok(false)
        } else {
            clipboard.set_contents(&updated_text)
        }
    } else {
        Ok(false)
    }
}

fn mangle_url(text: &str) -> String {
    match Url::parse(text) {
        Err(_) => text.to_owned(),
        Ok(mut url) => {
            if url.host_str() == Some("twitter.com") {
                let mut permissable_pairs = Vec::new();
                for (key, value) in url.query_pairs() {
                    if key != "t" && key != "s" {
                        permissable_pairs.push((key.into_owned(), value.into_owned()));
                    }
                }

                if permissable_pairs.is_empty() {
                    url.set_query(None);
                } else {
                    url.query_pairs_mut()
                        .clear()
                        .extend_pairs(permissable_pairs.iter())
                        .finish();
                }

                url.to_string()
            } else {
                text.to_owned()
            }
        }
    }
}
