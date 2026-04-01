mod default;
mod rainbow;

use crate::color::FlashColor;

pub use default::DefaultFlashEffect;
pub use rainbow::RainbowFlashEffect;

/// 针对某个已过时间点返回的一次效果采样结果。
///
/// `color` 表示该时刻应使用的闪烁颜色。
/// `alpha` 使用归一化范围 `0.0..=1.0` 表示不透明度。
/// `next_step_ms` 用来表示下一次采样的调度信息：
/// `Some(ms)` 表示在 `ms` 毫秒后继续采样，`None` 表示动画结束。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlashSample {
    /// 当前帧应使用的闪烁颜色。
    pub color: FlashColor,
    /// 使用归一化范围 `0.0..=1.0` 表示的不透明度（alpha）。
    pub alpha: f32,
    /// 下一次采样前需要等待的毫秒数，`None` 表示结束。
    pub next_step_ms: Option<u64>,
}

/// 为屏幕闪烁效果提供采样结果。
///
/// 实现者会接收自闪烁开始以来已经过去的毫秒数，
/// 并返回该时刻应使用的颜色、不透明度以及下一次采样的调度信息。
pub trait FlashEffect {
    /// 返回给定已过时间对应的采样结果。
    fn sample(&self, elapsed_ms: u64) -> FlashSample;
}
