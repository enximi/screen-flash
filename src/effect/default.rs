use crate::color::FlashColor;

use super::{FlashEffect, FlashSample};

/// 内置默认效果：先短暂保持较高亮度，再平滑淡出。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultFlashEffect {
    /// 默认效果使用的颜色。
    pub color: FlashColor,
}

impl Default for DefaultFlashEffect {
    fn default() -> Self {
        Self {
            color: FlashColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        }
    }
}

impl FlashEffect for DefaultFlashEffect {
    fn sample(&self, elapsed_ms: u64) -> FlashSample {
        const HOLD_MS: u64 = 90;
        const FADE_MS: u64 = 240;
        const DURATION_MS: u64 = HOLD_MS + FADE_MS;
        const MAX_ALPHA: f32 = 0.6;
        const STEP_MS: u64 = 16;

        let alpha = if elapsed_ms <= HOLD_MS {
            MAX_ALPHA
        } else if elapsed_ms >= DURATION_MS {
            0.0
        } else {
            let fade_elapsed_ms = elapsed_ms.saturating_sub(HOLD_MS);
            let remaining_ms = FADE_MS.saturating_sub(fade_elapsed_ms);
            MAX_ALPHA * (remaining_ms as f32 / FADE_MS as f32)
        };

        FlashSample {
            color: self.color,
            alpha,
            next_step_ms: if elapsed_ms >= DURATION_MS {
                None
            } else {
                Some(STEP_MS)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_effect_holds_then_fades() {
        let effect = DefaultFlashEffect {
            color: FlashColor {
                red: 12,
                green: 34,
                blue: 56,
            },
        };
        let start = effect.sample(0);
        let hold = effect.sample(45);
        let fading = effect.sample(210);
        let finished = effect.sample(10_000);

        assert_eq!(start.color, effect.color);
        assert_eq!(hold.alpha, start.alpha);
        assert!(fading.alpha < start.alpha);
        assert!(fading.alpha > 0.0);
        assert_eq!(finished.alpha, 0.0);
    }

    #[test]
    fn default_effect_reports_schedule_while_active_and_stops_when_finished() {
        let effect = DefaultFlashEffect {
            color: FlashColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        };
        let active = effect.sample(0);
        let finished = effect.sample(10_000);

        assert!(matches!(active.next_step_ms, Some(step_ms) if step_ms > 0));
        assert_eq!(finished.next_step_ms, None);
    }

    #[test]
    fn default_effect_uses_black_as_default_color() {
        assert_eq!(
            DefaultFlashEffect::default().color,
            FlashColor {
                red: 0,
                green: 0,
                blue: 0,
            }
        );
    }
}
