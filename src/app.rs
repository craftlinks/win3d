use std::time::{SystemTime, Instant};

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
};
pub type Result<T> = core::result::Result<T, Win32Error>;
use crate::{window::Window, error::Win32Error};

pub struct App {
    window: Window,
    init_time: Instant,
}

impl App {
    pub fn new() -> App {
        App {
            window: Window::new(800, 600, "-"),
            init_time: Instant::now(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.window.initialize()?;
        let mut message = MSG::default();
        loop {
            unsafe {
                // Initially the window is not visible
                if self.window.visible {
                    
                    while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).into() {
                        if message.message == WM_QUIT {
                            return Ok(());
                        }
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                    self.render()?;
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
        let now = Instant::now().duration_since(self.init_time).as_secs_f32();
        let c = f32::sin(now) / 2.0 + 0.5;
        self.window.gfx.as_mut().unwrap().clear_buffer(c / 1.2, 1.0 - c / 1.5,  1.0 - c /1.2);
        self.window.gfx.as_ref().unwrap().present_frame()?;
        Ok(())
    }
}
