mod color;
mod flash;

use windows::core::Result;

pub use color::FlashColor;

pub fn flash_screen(color: FlashColor) -> Result<()> {
    flash::flash_screen(color)
}
