use std::{intrinsics::transmute, mem::size_of};

use once_cell::sync::Lazy;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{LoadIconW, HICON},
        },
    },
};

use crate::{window::Window, Win32Result, WM_APP_TRAYMSG};

const NOTIFICATION_ID: u32 = 0xDEADBEEF;

static ICON: Lazy<HICON> =
    Lazy::new(|| unsafe { LoadIconW(GetModuleHandleW(None), MAKEINTRESOURCEW(69)) });

fn create_notification_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        uID: NOTIFICATION_ID,
        hWnd: hwnd,
        hIcon: *ICON,
        uFlags: NIF_ICON | NIF_MESSAGE,
        uCallbackMessage: WM_APP_TRAYMSG,
        ..Default::default()
    }
}

#[allow(non_snake_case)]
unsafe fn MAKEINTRESOURCEW(res: u16) -> PCWSTR {
    transmute(res as usize)
}

pub struct TrayIcon {
    hwnd: HWND,
}

impl TryFrom<&Window> for TrayIcon {
    type Error = windows::core::Error;

    fn try_from(value: &Window) -> Win32Result<Self> {
        let hwnd = value.hwnd;
        let data = create_notification_data(hwnd);

        unsafe { Shell_NotifyIconW(NIM_ADD, &data) }.ok()?;

        Ok(Self { hwnd })
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let data = create_notification_data(self.hwnd);

        unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
    }
}
