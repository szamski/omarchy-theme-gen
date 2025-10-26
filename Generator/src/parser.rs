use crate::color::{Color, ColorPalette};
use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse alacritty.toml format
pub fn parse_alacritty(path: &Path) -> Result<ColorPalette> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read alacritty file: {:?}", path))?;

    let value: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse alacritty TOML: {:?}", path))?;

    let mut palette = ColorPalette::default();

    // Extract colors from [colors.primary]
    if let Some(primary) = value
        .get("colors")
        .and_then(|c| c.get("primary"))
        .and_then(|p| p.as_table())
    {
        if let Some(bg) = primary.get("background").and_then(|v| v.as_str()) {
            palette.background = Color::new(bg).ok();
        }
        if let Some(fg) = primary.get("foreground").and_then(|v| v.as_str()) {
            palette.foreground = Color::new(fg).ok();
        }
    }

    // Extract colors from [colors.normal]
    if let Some(normal) = value
        .get("colors")
        .and_then(|c| c.get("normal"))
        .and_then(|n| n.as_table())
    {
        palette.black = normal.get("black").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.red = normal.get("red").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.green = normal.get("green").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.yellow = normal.get("yellow").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.blue = normal.get("blue").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.magenta = normal.get("magenta").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.cyan = normal.get("cyan").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.white = normal.get("white").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
    }

    // Extract colors from [colors.bright]
    if let Some(bright) = value
        .get("colors")
        .and_then(|c| c.get("bright"))
        .and_then(|b| b.as_table())
    {
        palette.bright_black = bright.get("black").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_red = bright.get("red").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_green = bright.get("green").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_yellow = bright.get("yellow").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_blue = bright.get("blue").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_magenta = bright.get("magenta").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_cyan = bright.get("cyan").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
        palette.bright_white = bright.get("white").and_then(|v| v.as_str()).and_then(|s| Color::new(s).ok());
    }

    // Extract cursor colors
    if let Some(cursor) = value
        .get("colors")
        .and_then(|c| c.get("cursor"))
        .and_then(|c| c.as_table())
    {
        if let Some(cursor_color) = cursor.get("cursor").and_then(|v| v.as_str()) {
            palette.cursor = Color::new(cursor_color).ok();
        }
    }

    // Extract selection colors
    if let Some(selection) = value
        .get("colors")
        .and_then(|c| c.get("selection"))
        .and_then(|s| s.as_table())
    {
        palette.selection_background = selection
            .get("background")
            .and_then(|v| v.as_str())
            .and_then(|s| Color::new(s).ok());
        palette.selection_foreground = selection
            .get("foreground")
            .and_then(|v| v.as_str())
            .and_then(|s| Color::new(s).ok());
    }

    Ok(palette)
}

/// Parse btop.theme format (key-value pairs with theme[key]="value")
pub fn parse_btop(path: &Path) -> Result<ColorPalette> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read btop file: {:?}", path))?;

    let mut palette = ColorPalette::default();
    let re = Regex::new(r#"theme\[(\w+)\]="(#[0-9a-fA-F]{6})""#)?;

    for cap in re.captures_iter(&content) {
        let key = &cap[1];
        let color_str = &cap[2];

        if let Ok(color) = Color::new(color_str) {
            match key {
                "main_bg" => palette.background = Some(color),
                "main_fg" => palette.foreground = Some(color),
                _ => {
                    palette.custom.insert(key.to_string(), color);
                }
            }
        }
    }

    Ok(palette)
}

/// Parse custom_theme.json format
#[derive(Debug, Deserialize)]
struct CustomThemeJson {
    colors: HashMap<String, String>,
}

pub fn parse_custom_json(path: &Path) -> Result<ColorPalette> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read custom theme JSON: {:?}", path))?;

    let theme: CustomThemeJson = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse custom theme JSON: {:?}", path))?;

    let mut palette = ColorPalette::default();

    for (key, value) in theme.colors {
        if let Ok(color) = Color::new(&value) {
            match key.as_str() {
                "background" => palette.background = Some(color),
                "foreground" => palette.foreground = Some(color),
                "black" => palette.black = Some(color),
                "red" => palette.red = Some(color),
                "green" => palette.green = Some(color),
                "yellow" => palette.yellow = Some(color),
                "blue" => palette.blue = Some(color),
                "magenta" => palette.magenta = Some(color),
                "cyan" => palette.cyan = Some(color),
                "white" => palette.white = Some(color),
                "bright_black" => palette.bright_black = Some(color),
                "bright_red" => palette.bright_red = Some(color),
                "bright_green" => palette.bright_green = Some(color),
                "bright_yellow" => palette.bright_yellow = Some(color),
                "bright_blue" => palette.bright_blue = Some(color),
                "bright_magenta" => palette.bright_magenta = Some(color),
                "bright_cyan" => palette.bright_cyan = Some(color),
                "bright_white" => palette.bright_white = Some(color),
                "cursor" => palette.cursor = Some(color),
                "selection_background" => palette.selection_background = Some(color),
                "selection_foreground" => palette.selection_foreground = Some(color),
                _ => {
                    palette.custom.insert(key, color);
                }
            }
        }
    }

    Ok(palette)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_btop() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "theme[main_bg]=\"#eff1f5\"").unwrap();
        writeln!(file, "theme[main_fg]=\"#4c4f69\"").unwrap();
        writeln!(file, "theme[title]=\"#dc8a78\"").unwrap();
        file.flush().unwrap();

        let palette = parse_btop(file.path()).unwrap();
        assert!(palette.background.is_some());
        assert!(palette.foreground.is_some());
        assert_eq!(palette.custom.get("title").unwrap().hex(), "#dc8a78");
    }

    #[test]
    fn test_parse_custom_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "{{\"colors\": {{\"background\": \"#eff1f5\", \"foreground\": \"#4c4f69\"}}}}"
        )
        .unwrap();
        file.flush().unwrap();

        let palette = parse_custom_json(file.path()).unwrap();
        assert!(palette.background.is_some());
        assert!(palette.foreground.is_some());
    }
}
