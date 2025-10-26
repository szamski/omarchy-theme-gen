use crate::color::ColorPalette;
use crate::parser;
use anyhow::Result;
use std::path::Path;
use tracing::{debug, info, warn};

/// Source of color extraction
#[derive(Debug, Clone, Copy)]
pub enum ColorSource {
    Alacritty,
    Btop,
    CustomJson,
}

impl ColorSource {
    pub fn filename(&self) -> &'static str {
        match self {
            ColorSource::Alacritty => "alacritty.toml",
            ColorSource::Btop => "btop.theme",
            ColorSource::CustomJson => "custom_theme.json",
        }
    }
}

/// Extract color palette from a theme directory
/// Tries sources in priority order until successful
pub fn extract_colors(
    theme_dir: &Path,
    priority: &[ColorSource],
) -> Result<(ColorPalette, ColorSource)> {
    let mut combined_palette = ColorPalette::default();
    let mut primary_source = None;

    for source in priority {
        let file_path = theme_dir.join(source.filename());

        if !file_path.exists() {
            debug!(
                "Color source {:?} not found at {:?}",
                source,
                file_path
            );
            continue;
        }

        info!("Attempting to extract colors from {:?}", file_path);

        let palette = match source {
            ColorSource::Alacritty => parser::parse_alacritty(&file_path),
            ColorSource::Btop => parser::parse_btop(&file_path),
            ColorSource::CustomJson => parser::parse_custom_json(&file_path),
        };

        match palette {
            Ok(palette) => {
                info!(
                    "Successfully extracted colors from {:?} ({:?})",
                    file_path, source
                );

                // First successful source becomes primary
                if primary_source.is_none() {
                    primary_source = Some(*source);
                }

                // Merge colors into combined palette
                combined_palette.merge(palette);
            }
            Err(e) => {
                warn!(
                    "Failed to parse {:?}: {}",
                    file_path,
                    e
                );
            }
        }
    }

    // Check if we extracted any colors
    if primary_source.is_none() {
        anyhow::bail!(
            "No color sources found in theme directory: {:?}",
            theme_dir
        );
    }

    // Fallback logic: use reasonable defaults if critical colors are missing
    if combined_palette.background.is_none() {
        warn!("No background color found, using default #ffffff");
        combined_palette.background = Some(crate::color::Color::new("#ffffff")?);
    }

    if combined_palette.foreground.is_none() {
        warn!("No foreground color found, using default #000000");
        combined_palette.foreground = Some(crate::color::Color::new("#000000")?);
    }

    Ok((combined_palette, primary_source.unwrap()))
}

/// Extract colors with default priority order
#[allow(dead_code)]
pub fn extract_colors_default(theme_dir: &Path) -> Result<(ColorPalette, ColorSource)> {
    let default_priority = vec![
        ColorSource::Alacritty,
        ColorSource::CustomJson,
        ColorSource::Btop,
    ];
    extract_colors(theme_dir, &default_priority)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_colors_from_alacritty() {
        let temp_dir = TempDir::new().unwrap();
        let alacritty_path = temp_dir.path().join("alacritty.toml");

        fs::write(
            &alacritty_path,
            "[colors.primary]\nbackground = \"#eff1f5\"\nforeground = \"#4c4f69\"\n\n[colors.normal]\nblack = \"#5c5f77\"\nred = \"#d20f39\"\n",
        )
        .unwrap();

        let (palette, source) = extract_colors_default(temp_dir.path()).unwrap();

        assert!(matches!(source, ColorSource::Alacritty));
        assert!(palette.background.is_some());
        assert!(palette.foreground.is_some());
        assert!(palette.black.is_some());
    }

    #[test]
    fn test_extract_colors_fallback_to_btop() {
        let temp_dir = TempDir::new().unwrap();
        let btop_path = temp_dir.path().join("btop.theme");

        fs::write(
            &btop_path,
            "theme[main_bg]=\"#eff1f5\"\ntheme[main_fg]=\"#4c4f69\"\n",
        )
        .unwrap();

        let (palette, source) = extract_colors_default(temp_dir.path()).unwrap();

        assert!(matches!(source, ColorSource::Btop));
        assert!(palette.background.is_some());
        assert!(palette.foreground.is_some());
    }
}
