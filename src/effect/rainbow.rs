use crate::color::FlashColor;

use super::{FlashEffect, FlashSample};

/// 内置彩虹效果：在闪烁过程中持续变换颜色，并以固定不透明度显示到结束。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RainbowFlashEffect;

impl FlashEffect for RainbowFlashEffect {
    fn sample(&self, elapsed_ms: u64) -> FlashSample {
        const DURATION_MS: u64 = 900;
        const ALPHA: f32 = 0.6;
        const STEP_MS: u64 = 16;

        if elapsed_ms >= DURATION_MS {
            return FlashSample {
                color: rainbow_color(DURATION_MS, DURATION_MS),
                alpha: 0.0,
                next_step_ms: None,
            };
        }

        FlashSample {
            color: rainbow_color(elapsed_ms, DURATION_MS),
            alpha: ALPHA,
            next_step_ms: Some(STEP_MS),
        }
    }
}

fn rainbow_color(elapsed_ms: u64, duration_ms: u64) -> FlashColor {
    let progress = (elapsed_ms.min(duration_ms) as f32 / duration_ms as f32).clamp(0.0, 1.0);
    hsv_to_rgb(progress * 360.0, 1.0, 1.0)
}

fn hsv_to_rgb(hue_deg: f32, saturation: f32, value: f32) -> FlashColor {
    let hue = hue_deg.rem_euclid(360.0);
    let chroma = value * saturation;
    let segment = hue / 60.0;
    let x = chroma * (1.0 - ((segment.rem_euclid(2.0)) - 1.0).abs());
    let (red, green, blue) = match segment as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let base = value - chroma;

    FlashColor {
        red: ((red + base) * 255.0).round() as u8,
        green: ((green + base) * 255.0).round() as u8,
        blue: ((blue + base) * 255.0).round() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::DefaultFlashEffect;

    #[test]
    fn rainbow_effect_changes_color_over_time() {
        let start = RainbowFlashEffect.sample(0);
        let middle = RainbowFlashEffect.sample(450);
        let finished = RainbowFlashEffect.sample(10_000);

        assert_ne!(start.color, middle.color);
        assert_eq!(start.alpha, middle.alpha);
        assert_eq!(finished.alpha, 0.0);
        assert_eq!(finished.next_step_ms, None);
    }

    #[test]
    fn rainbow_effect_lasts_longer_than_default_effect() {
        let default_finished = DefaultFlashEffect::default().sample(400);
        let rainbow_active = RainbowFlashEffect.sample(400);

        assert_eq!(default_finished.alpha, 0.0);
        assert!(rainbow_active.alpha > 0.0);
        assert!(rainbow_active.next_step_ms.is_some());
    }

    #[test]
    fn rainbow_effect_keeps_constant_alpha_while_active() {
        let start = RainbowFlashEffect.sample(0);
        let later = RainbowFlashEffect.sample(600);

        assert_eq!(start.alpha, later.alpha);
    }
}
