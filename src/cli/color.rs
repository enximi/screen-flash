use screen_flash::FlashColor;

pub fn parse_flash_color(value: &str) -> Result<FlashColor, String> {
    if value.contains(',') {
        return parse_rgb_color(value);
    }

    parse_hex_color(value)
}

fn parse_hex_color(value: &str) -> Result<FlashColor, String> {
    if value.len() != 6 {
        return Err("color must be in RRGGBB or r,g,b format".to_string());
    }

    let red = u8::from_str_radix(&value[0..2], 16)
        .map_err(|_| "color must be in RRGGBB or r,g,b format".to_string())?;
    let green = u8::from_str_radix(&value[2..4], 16)
        .map_err(|_| "color must be in RRGGBB or r,g,b format".to_string())?;
    let blue = u8::from_str_radix(&value[4..6], 16)
        .map_err(|_| "color must be in RRGGBB or r,g,b format".to_string())?;

    Ok(FlashColor { red, green, blue })
}

fn parse_rgb_color(value: &str) -> Result<FlashColor, String> {
    let parts = value.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("color must be in RRGGBB or r,g,b format".to_string());
    }

    let red = parts[0]
        .parse::<u8>()
        .map_err(|_| "each r,g,b value must be between 0 and 255".to_string())?;
    let green = parts[1]
        .parse::<u8>()
        .map_err(|_| "each r,g,b value must be between 0 and 255".to_string())?;
    let blue = parts[2]
        .parse::<u8>()
        .map_err(|_| "each r,g,b value must be between 0 and 255".to_string())?;

    Ok(FlashColor { red, green, blue })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_color() {
        assert_eq!(
            parse_flash_color("112233").unwrap(),
            FlashColor {
                red: 0x11,
                green: 0x22,
                blue: 0x33,
            }
        );
    }

    #[test]
    fn parses_rgb_color() {
        assert_eq!(
            parse_flash_color("12, 34, 56").unwrap(),
            FlashColor {
                red: 12,
                green: 34,
                blue: 56,
            }
        );
    }

    #[test]
    fn rejects_invalid_color() {
        assert!(parse_flash_color("fff").is_err());
        assert!(parse_flash_color("#aabbcc").is_err());
        assert!(parse_flash_color("#zzzzzz").is_err());
        assert!(parse_flash_color("1,2").is_err());
        assert!(parse_flash_color("256,0,0").is_err());
    }
}
