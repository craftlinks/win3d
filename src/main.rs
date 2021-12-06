#[macro_use]
mod error;
use error::Win32Error;
mod win32_common;


mod window;

use window::Window;
pub type Result<T> = core::result::Result<T, Win32Error>;

fn main() -> Result<()> {

    let mut wnd: Window = Window::new(800, 600, "Win3D");
    wnd.run()
}
