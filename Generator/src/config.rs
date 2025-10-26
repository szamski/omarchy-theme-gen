use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Configuration for the theme watcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to watch for theme changes (typically ~/.config/omarchy/current/theme)
    #[serde(default = "default_watch_path")]
    pub watch_path: PathBuf,

    /// External programs to generate configs for
    #[serde(default)]
    pub programs: Vec<ProgramConfig>,

    /// Priority order for color extraction sources
    #[serde(default = "default_color_priority")]
    pub color_priority: Vec<String>,

    /// Directory where generated theme files are stored
    #[serde(default = "default_generated_themes_dir")]
    pub generated_themes_dir: PathBuf,

    /// Automatically activate themes when generated
    #[serde(default = "default_true")]
    pub auto_activate: bool,

    /// Create backups of existing files before replacing with symlinks
    #[serde(default = "default_true")]
    pub create_backups: bool,

    /// Automatically create symlinks to theme directories
    #[serde(default = "default_true")]
    pub auto_symlink: bool,
}

fn default_watch_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/omarchy/current/theme")
}

fn default_color_priority() -> Vec<String> {
    vec![
        "alacritty.toml".to_string(),
        "custom_theme.json".to_string(),
        "btop.theme".to_string(),
    ]
}

fn default_generated_themes_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/omarchy-themes/generated")
}

/// Configuration for an external program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// Name of the program (e.g., "spicetify", "vencord")
    pub name: String,

    /// Whether this program is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Output filename to generate (e.g., "spicetify.ini", "vencord.theme.css")
    pub output_file: String,

    /// Template name to use
    pub template: String,

    /// Additional template variables (optional)
    #[serde(default)]
    pub variables: std::collections::HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Config {
            watch_path: default_watch_path(),
            programs: vec![
                ProgramConfig {
                    name: "omarcord".to_string(),
                    enabled: true,
                    output_file: "omarcord.theme.css".to_string(),
                    template: "omarcord".to_string(),
                    variables: std::collections::HashMap::new(),
                },
                ProgramConfig {
                    name: "omarchify".to_string(),
                    enabled: true,
                    output_file: "color.ini".to_string(),
                    template: "omarchify".to_string(),
                    variables: std::collections::HashMap::new(),
                },
            ],
            color_priority: default_color_priority(),
            generated_themes_dir: default_generated_themes_dir(),
            auto_activate: true,
            create_backups: true,
            auto_symlink: true,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            warn!(
                "Config file not found at {:?}, using defaults",
                path
            );
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        info!("Loaded configuration from {:?}", path);
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Get default config file path
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/omarchy-theme-watcher/config.toml")
    }

    /// Load config from default location or create if it doesn't exist
    pub fn load_or_create_default() -> Result<Self> {
        let path = Self::default_path();

        if path.exists() {
            Self::load(&path)
        } else {
            info!("Creating default config at {:?}", path);
            let config = Config::default();
            config.save(&path)?;
            Ok(config)
        }
    }

    /// Get enabled programs
    pub fn enabled_programs(&self) -> impl Iterator<Item = &ProgramConfig> {
        self.programs.iter().filter(|p| p.enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.programs.len() > 0);
        assert!(config.color_priority.contains(&"alacritty.toml".to_string()));
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::default();

        config.save(temp_file.path()).unwrap();
        let loaded = Config::load(temp_file.path()).unwrap();

        assert_eq!(config.programs.len(), loaded.programs.len());
    }
}
