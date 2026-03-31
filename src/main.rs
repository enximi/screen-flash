use std::error::Error;
use std::fmt::{Display, Formatter};

use clap::Parser;

use screen_flash::{FlashColor, flash_screen};

#[derive(Parser, Debug)]
#[command(author, version, about = "Flash the full screen with a custom color.")]
struct Cli {
    /// Flash color as #RRGGBB, RRGGBB, or r,g,b
    color: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let color = parse_color(&cli.color)?;

    flash_screen(color)?;

    Ok(())
}

fn parse_color(input: &str) -> Result<FlashColor, CliError> {
    if let Some(hex) = input.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    if input.contains(',') {
        return parse_rgb_triplet(input);
    }

    parse_hex_color(input)
}

fn parse_hex_color(input: &str) -> Result<FlashColor, CliError> {
    if input.len() != 6 {
        return Err(CliError::InvalidColor(input.to_owned()));
    }

    let red = parse_hex_channel(&input[0..2], input)?;
    let green = parse_hex_channel(&input[2..4], input)?;
    let blue = parse_hex_channel(&input[4..6], input)?;

    Ok(FlashColor { red, green, blue })
}

fn parse_hex_channel(channel: &str, source: &str) -> Result<u8, CliError> {
    u8::from_str_radix(channel, 16).map_err(|_| CliError::InvalidColor(source.to_owned()))
}

fn parse_rgb_triplet(input: &str) -> Result<FlashColor, CliError> {
    let parts: Vec<_> = input.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return Err(CliError::InvalidColor(input.to_owned()));
    }

    let red = parse_u8_channel(parts[0], input)?;
    let green = parse_u8_channel(parts[1], input)?;
    let blue = parse_u8_channel(parts[2], input)?;

    Ok(FlashColor { red, green, blue })
}

fn parse_u8_channel(channel: &str, source: &str) -> Result<u8, CliError> {
    channel
        .parse::<u8>()
        .map_err(|_| CliError::InvalidColor(source.to_owned()))
}

#[derive(Debug)]
enum CliError {
    InvalidColor(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidColor(color) => write!(
                f,
                "invalid color `{color}`; expected #RRGGBB, RRGGBB, or r,g,b"
            ),
        }
    }
}

impl Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hash_prefixed_hex_color() {
        assert_eq!(
            parse_color("#0C2238").unwrap(),
            FlashColor {
                red: 12,
                green: 34,
                blue: 56,
            }
        );
    }

    #[test]
    fn parses_plain_hex_color() {
        assert_eq!(
            parse_color("0c2238").unwrap(),
            FlashColor {
                red: 12,
                green: 34,
                blue: 56,
            }
        );
    }

    #[test]
    fn parses_rgb_triplet_color() {
        assert_eq!(
            parse_color("12, 34, 56").unwrap(),
            FlashColor {
                red: 12,
                green: 34,
                blue: 56,
            }
        );
    }

    #[test]
    fn rejects_invalid_color() {
        assert!(matches!(
            parse_color("oops"),
            Err(CliError::InvalidColor(_))
        ));
    }
}
