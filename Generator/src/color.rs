use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Represents a hex color value
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub String);

impl Color {
    /// Create a new color from a hex string (with or without #)
    pub fn new(hex: impl Into<String>) -> Result<Self> {
        let hex: String = hex.into();
        let hex = hex.trim_start_matches('#');

        // Validate hex format (3 or 6 characters)
        if hex.len() != 3 && hex.len() != 6 {
            anyhow::bail!("Invalid hex color length: {}", hex);
        }

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("Invalid hex color format: {}", hex);
        }

        // Normalize to 6-character format
        let normalized = if hex.len() == 3 {
            hex.chars()
                .flat_map(|c| std::iter::repeat(c).take(2))
                .collect()
        } else {
            hex.to_string()
        };

        Ok(Color(format!("#{}", normalized)))
    }

    /// Get hex value with #
    pub fn hex(&self) -> &str {
        &self.0
    }

    /// Get hex value without #
    #[allow(dead_code)]
    pub fn hex_no_hash(&self) -> &str {
        self.0.trim_start_matches('#')
    }

    /// Convert to RGB values (r, g, b) where each is 0-255
    #[allow(dead_code)]
    pub fn to_rgb(&self) -> Result<(u8, u8, u8)> {
        let hex = self.hex_no_hash();
        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        Ok((r, g, b))
    }

    /// Convert to RGB string "r, g, b"
    #[allow(dead_code)]
    pub fn to_rgb_string(&self) -> Result<String> {
        let (r, g, b) = self.to_rgb()?;
        Ok(format!("{}, {}, {}", r, g, b))
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Color::new(s)
    }
}

/// Standard color palette extracted from theme files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorPalette {
    // Base colors
    pub background: Option<Color>,
    pub foreground: Option<Color>,

    // Normal colors (ANSI 0-7)
    pub black: Option<Color>,
    pub red: Option<Color>,
    pub green: Option<Color>,
    pub yellow: Option<Color>,
    pub blue: Option<Color>,
    pub magenta: Option<Color>,
    pub cyan: Option<Color>,
    pub white: Option<Color>,

    // Bright colors (ANSI 8-15)
    pub bright_black: Option<Color>,
    pub bright_red: Option<Color>,
    pub bright_green: Option<Color>,
    pub bright_yellow: Option<Color>,
    pub bright_blue: Option<Color>,
    pub bright_magenta: Option<Color>,
    pub bright_cyan: Option<Color>,
    pub bright_white: Option<Color>,

    // Additional common colors
    pub cursor: Option<Color>,
    pub selection_background: Option<Color>,
    pub selection_foreground: Option<Color>,

    // Store any additional custom colors
    pub custom: HashMap<String, Color>,
}

impl ColorPalette {
    /// Extract hex colors from text using regex
    #[allow(dead_code)]
    pub fn extract_hex_colors(text: &str) -> Result<Vec<Color>> {
        let re = Regex::new(r#"#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b"#)?;
        let mut colors = Vec::new();

        for cap in re.captures_iter(text) {
            if let Some(hex) = cap.get(0) {
                if let Ok(color) = Color::new(hex.as_str()) {
                    colors.push(color);
                }
            }
        }

        Ok(colors)
    }

    /// Merge another palette into this one, keeping existing values
    pub fn merge(&mut self, other: ColorPalette) {
        macro_rules! merge_field {
            ($field:ident) => {
                if self.$field.is_none() && other.$field.is_some() {
                    self.$field = other.$field;
                }
            };
        }

        merge_field!(background);
        merge_field!(foreground);
        merge_field!(black);
        merge_field!(red);
        merge_field!(green);
        merge_field!(yellow);
        merge_field!(blue);
        merge_field!(magenta);
        merge_field!(cyan);
        merge_field!(white);
        merge_field!(bright_black);
        merge_field!(bright_red);
        merge_field!(bright_green);
        merge_field!(bright_yellow);
        merge_field!(bright_blue);
        merge_field!(bright_magenta);
        merge_field!(bright_cyan);
        merge_field!(bright_white);
        merge_field!(cursor);
        merge_field!(selection_background);
        merge_field!(selection_foreground);

        // Merge custom colors
        for (key, value) in other.custom {
            self.custom.entry(key).or_insert(value);
        }
    }

    /// Get a color by name (checking both standard and custom colors)
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&Color> {
        match name {
            "background" => self.background.as_ref(),
            "foreground" => self.foreground.as_ref(),
            "black" => self.black.as_ref(),
            "red" => self.red.as_ref(),
            "green" => self.green.as_ref(),
            "yellow" => self.yellow.as_ref(),
            "blue" => self.blue.as_ref(),
            "magenta" => self.magenta.as_ref(),
            "cyan" => self.cyan.as_ref(),
            "white" => self.white.as_ref(),
            "bright_black" => self.bright_black.as_ref(),
            "bright_red" => self.bright_red.as_ref(),
            "bright_green" => self.bright_green.as_ref(),
            "bright_yellow" => self.bright_yellow.as_ref(),
            "bright_blue" => self.bright_blue.as_ref(),
            "bright_magenta" => self.bright_magenta.as_ref(),
            "bright_cyan" => self.bright_cyan.as_ref(),
            "bright_white" => self.bright_white.as_ref(),
            "cursor" => self.cursor.as_ref(),
            "selection_background" => self.selection_background.as_ref(),
            "selection_foreground" => self.selection_foreground.as_ref(),
            _ => self.custom.get(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new("#ff0000").unwrap();
        assert_eq!(color.hex(), "#ff0000");

        let color = Color::new("00ff00").unwrap();
        assert_eq!(color.hex(), "#00ff00");

        let color = Color::new("#f00").unwrap();
        assert_eq!(color.hex(), "#ff0000");
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::new("#ff0000").unwrap();
        assert_eq!(color.to_rgb().unwrap(), (255, 0, 0));

        let color = Color::new("#00ff00").unwrap();
        assert_eq!(color.to_rgb().unwrap(), (0, 255, 0));
    }

    #[test]
    fn test_extract_hex_colors() {
        let text = "background = \"#ff0000\" foreground = \"#00ff00\"";
        let colors = ColorPalette::extract_hex_colors(text).unwrap();
        assert_eq!(colors.len(), 2);
    }
}
