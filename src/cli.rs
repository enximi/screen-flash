mod color;

use clap::{Parser, ValueEnum};
use screen_flash::{DefaultFlashEffect, FlashColor, RainbowFlashEffect, flash_screen};

use self::color::parse_flash_color;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum BuiltinEffect {
    Default,
    Rainbow,
}

#[derive(Debug, Parser)]
#[command(author, version, about = "Play a flash effect on the screen")]
struct Cli {
    /// Select a built-in flash effect.
    #[arg(value_enum, default_value_t = BuiltinEffect::Default)]
    effect: BuiltinEffect,

    /// Only valid for the default effect. Format: RRGGBB or r,g,b.
    #[arg(long, value_parser = parse_flash_color)]
    color: Option<FlashColor>,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    Cli::parse().run()
}

impl Cli {
    fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.effect {
            BuiltinEffect::Default => flash_screen(
                self.color
                    .map(|color| DefaultFlashEffect { color })
                    .unwrap_or_default(),
            )?,
            BuiltinEffect::Rainbow => {
                if self.color.is_some() {
                    return Err("--color can only be used with the default effect".into());
                }

                flash_screen(RainbowFlashEffect)?;
            }
        }

        Ok(())
    }
}
