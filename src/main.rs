use windows::{
    core::Result};

mod win32_common;


mod window;
use window::Window;


fn main() -> Result<()> {

    let mut wnd: Window = Window::new(800, 600, "Win3D");
    wnd.run()
}