use std::mem::{size_of, transmute};

use once_cell::sync::Lazy;
use windows::Win32::{
    Foundation::{HWND, PWSTR},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Shell::{Shell_NotifyIconW, NIF_ICON, NOTIFYICONDATAW},
        WindowsAndMessaging::{LoadIconW, HICON},
    },
};

use crate::{window::Window, Win32Result};

const NIM_ADD: u32 = 0x0;
const NIM_DELETE: u32 = 0x2;

const NOTIFICATION_ID: u32 = 0xDEADBEEF;

static ICON: Lazy<HICON> =
    Lazy::new(|| unsafe { LoadIconW(GetModuleHandleW(None), makeintresourcew(69)) });

fn create_notification_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        uID: NOTIFICATION_ID,
        hWnd: hwnd,
        hIcon: *ICON,
        uFlags: NIF_ICON,
        ..Default::default()
    }
}

unsafe fn makeintresourcew(res: u16) -> PWSTR {
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
