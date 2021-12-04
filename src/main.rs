use windows::{
    core::Result};

mod window;
use window::Window;
fn main() -> Result<()> {

    let wnd: Window = Window::new(800, 300, "Main application window");
    wnd.run()
}