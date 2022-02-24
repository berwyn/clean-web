#![windows_subsystem = "windows"]

use clipboard::Clipboard;
use config::Config;
use menu::MenuOptions;
use once_cell::sync::Lazy;
use tray::TrayIcon;
use url::Url;
use window::Window;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        DefWindowProcW, DestroyWindow, PostQuitMessage, WM_APP, WM_CLIPBOARDUPDATE, WM_CLOSE,
        WM_DESTROY, WM_RBUTTONUP,
    },
};

mod clipboard;
mod config;
mod menu;
mod tray;
mod util;
mod window;

const WINDOW_CLASS_NAME: &str = "CleanWeb";
const WM_APP_TRAYMSG: u32 = WM_APP + 1;

pub type Win32Result<T> = Result<T, windows::core::Error>;

static CONFIG: Lazy<Config> = Lazy::new(|| {
    let path = Config::csv_path().expect("Config path should be valid");
    path.try_into().unwrap_or_default()
});

fn main() {
    CONFIG.ensure_exists().expect("Unable to load config");

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
        WM_APP_TRAYMSG => match lparam.0 as u32 {
            WM_RBUTTONUP => {
                match menu::show_menu(hwnd) {
                    Some(MenuOptions::Exit) => {
                        DestroyWindow(hwnd);
                    }
                    None => {}
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        },
        WM_CLOSE => {
            DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
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
            if !url.has_host() {
                return url.to_string();
            }

            let host = url.host().unwrap().to_string();

            for (host_regex, param_regex) in CONFIG.iter() {
                if host_regex.is_match(&host) {
                    let mut permissable_pairs = Vec::new();

                    for (key, value) in url.query_pairs() {
                        if !param_regex.is_match(&key) {
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
                }
            }

            url.to_string()
        }
    }
}
