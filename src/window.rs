use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::Result;
use windows::Win32::Foundation::{HWND, PWSTR};
use windows::Win32::UI::WindowsAndMessaging::{LoadCursorW, CS_HREDRAW, CS_VREDRAW, IDC_HAND, WNDCLASSW};

pub struct Window {
    width: u32,
    height: u32,
    window_name: String,
    window_handle: HWND,
}

impl Window {
    pub fn new(width: u32, height: u32, window_user_name: &str) -> Window {
        Window {
            width,
            height,
            window_name: window_user_name.into(),
            window_handle: HWND(0), // will need to be set on actual window creation
        }
    }

    pub fn run(&mut self) -> Result<()> {
        unsafe {
            let wc = {
                WNDCLASSW {
                    hCursor: LoadCursorW(None, IDC_HAND),
                    hInstance: GetModuleHandleW(None),
                    lpszClassName: PWSTR(b"window\0".as_ptr() as _), // Geert, will need to be fixed, look at Minecraft example.

                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(Self::wndproc), // Todo 
                    ..Default::default()
                }
            };
            Ok(())
        }
    }
}
