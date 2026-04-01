//! 一个面向 Windows 的屏幕闪烁库。
//!
//! 调用方通过实现 [`FlashEffect`]，为每个采样时刻提供颜色、alpha 和下一次调度时间，
//! 库则负责创建全屏覆盖窗口并把这些采样结果渲染到屏幕上。

#[cfg(not(windows))]
compile_error!("screen-flash only supports Windows.");

mod color;
mod effect;
mod flash;

pub use color::FlashColor;
pub use effect::RainbowFlashEffect;
pub use effect::{DefaultFlashEffect, FlashEffect, FlashSample};
pub use flash::flash_screen;
