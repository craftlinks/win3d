use crate::win32_common::ToWide;
use std::ffi::c_void;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRect, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    GetMessageW, GetWindowLongPtrW, LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassW,
    SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWLP_USERDATA, IDC_CROSS, MSG, PM_REMOVE, WM_ACTIVATE, WM_DESTROY, WM_NCCREATE, WM_QUIT,
    WNDCLASSW, WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPEDWINDOW, WS_SYSMENU, WS_VISIBLE,
};

// Dealing with errors
//======================
// .map_err(|e| os_error!(e));
// return Err(os_error!(::windows::core::Error::from_win32()))
// For example: AdjustWindowRect(&mut wr, WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU, BOOL(0)).ok().map_err(|e| win_error!(e))?;
use crate::error::Win32Error;
pub type Result<T> = core::result::Result<T, Win32Error>;


pub struct Window {
    width: i32,
    height: i32,
    window_name: String,
    window_handle: HWND,
    visible: bool,
}

impl Window {
    pub fn new(width: i32, height: i32, window_user_name: &str) -> Window {
        Window {
            width,
            height,
            window_name: window_user_name.into(),
            window_handle: HWND(0),
            visible: false, // will need to be set on actual window creation
        }
    }

    pub fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleW(None);
            let window_class_name = "window".to_wide().as_ptr() as *mut u16;

            let wc = {
                WNDCLASSW {
                    hCursor: LoadCursorW(None, IDC_CROSS),
                    hInstance: instance,
                    lpszClassName: PWSTR(window_class_name),

                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(Self::wndproc),
                    ..Default::default()
                }
            };

            let atom = RegisterClassW(&wc);
            debug_assert!(atom != 0);

            let window_handle = {
                // calculate window size based on desired client region size
                let mut wr = RECT::default();
                wr.left = 100;
                wr.right = self.width + wr.left;
                wr.top = 100;
                wr.bottom = self.height + wr.top;
                // Adjust window size to accomodate the desired client dimensions specified by `width` and `height`.
                AdjustWindowRect(&mut wr, WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU, BOOL(0)).ok().map_err(|e| win_error!(e))?;
                let window_name: &str = &self.window_name;
                CreateWindowExW(
                    Default::default(),
                    PWSTR(window_class_name),
                    PWSTR(window_name.to_wide().as_ptr() as *mut u16),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    wr.right - wr.left,
                    wr.bottom - wr.top,
                    None,
                    None,
                    instance,
                    self as *mut Window as *const c_void,
                )
            };

            debug_assert!(window_handle.0 != 0);
            debug_assert!(window_handle == self.window_handle);
            let mut message = MSG::default();

            loop {
                // Initially the window is not visible
                if self.visible {
                    self.render()?;

                    while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).into() {
                        if message.message == WM_QUIT {
                            return Ok(());
                        }
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                } else {
                    GetMessageW(&mut message, None, 0, 0);

                    if message.message == WM_QUIT {
                        return Ok(());
                    }
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        Ok(())
    }

    fn user_message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_ACTIVATE => {
                    self.visible = true; // TODO: unpack !HIWORD(wparam);
                    LRESULT(0)
                }

                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(self.window_handle, message, wparam, lparam),
            }
        }
    }

    extern "system" fn wndproc(
        window_handle: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam.0 as *const CREATESTRUCTW;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).window_handle = window_handle;
                SetWindowLongPtrW(window_handle, GWLP_USERDATA, this as isize);
            } else {
                let this = GetWindowLongPtrW(window_handle, GWLP_USERDATA) as *mut Self;
                if !this.is_null() {
                    return (*this).user_message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcW(window_handle, message, wparam, lparam)
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.window_handle.0 != 0 {
                println!("Destroying window.");
                let _ = DestroyWindow(self.window_handle).ok().map_err(|e| println!("{}",win_error!(e)));
            }
        }
    }
}
