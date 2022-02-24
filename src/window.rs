use std::mem::size_of;

use windows::Win32::{
    Foundation::HWND,
    System::DataExchange::AddClipboardFormatListener,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, GetMessageW, RegisterClassExW, TranslateMessage,
        HWND_MESSAGE, MSG, WNDCLASSEXW, WNDPROC,
    },
};

use crate::{util::string_to_pcwstr, Win32Result};

pub struct Window {
    pub hwnd: HWND,
}

impl Window {
    pub fn new(class_name: &str, handler: WNDPROC) -> Win32Result<Self> {
        let wnd_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpszClassName: string_to_pcwstr(class_name),
            lpfnWndProc: handler,
            ..Default::default()
        };

        let result = unsafe { RegisterClassExW(&wnd_class) };

        if result == 0 {
            todo!("Couldn't register class")
        }

        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                class_name,
                class_name,
                Default::default(),
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                None,
                None,
                std::ptr::null(),
            )
        };

        Ok(Self { hwnd })
    }

    pub fn listen_to_clipboard(&self) -> Win32Result<()> {
        unsafe { AddClipboardFormatListener(self.hwnd) }.ok()
    }

    pub fn run_message_loop(&self) {
        loop {
            let mut msg = MSG::default();
            let has_message = unsafe { GetMessageW(&mut msg, HWND(0), 1, 0) };
            if has_message.as_bool() {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            } else {
                break;
            }
        }
    }
}
