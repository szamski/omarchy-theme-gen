use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;
use tracing::{debug, info, warn};

use crate::detector::InstalledProgram;

/// Result of a theme activation
#[derive(Debug)]
pub struct ActivationResult {
    pub program: String,
    pub success: bool,
    pub message: String,
}

/// Handles activation of themes for various programs
pub struct ThemeActivator;

impl ThemeActivator {
    /// Activate theme for all detected programs
    pub fn activate_all(programs: &[InstalledProgram]) -> Vec<ActivationResult> {
        let mut results = Vec::new();

        for program in programs {
            let result = match program.name.as_str() {
                "vencord" => Self::activate_vencord(program),
                "spicetify" => Self::activate_spicetify(program),
                _ => Ok(ActivationResult {
                    program: program.name.clone(),
                    success: false,
                    message: format!("Activation not supported for {}", program.name),
                }),
            };

            match result {
                Ok(r) => results.push(r),
                Err(e) => results.push(ActivationResult {
                    program: program.name.clone(),
                    success: false,
                    message: format!("Error: {:#}", e),
                }),
            }
        }

        results
    }

    /// Activate Omarcord theme (wrapper for Vencord with correct theme name)
    pub fn activate_omarcord(program: &InstalledProgram) -> Result<ActivationResult> {
        Self::activate_vencord_with_name(program, "omarcord.theme.css")
    }

    /// Activate Omarchify theme (wrapper for Spicetify with correct scheme name)
    pub fn activate_omarchify(program: &InstalledProgram) -> Result<ActivationResult> {
        Self::activate_spicetify_with_scheme(program, "text", "Omarchify")
    }

    /// Activate Vencord theme by updating settings.json
    fn activate_vencord(program: &InstalledProgram) -> Result<ActivationResult> {
        Self::activate_vencord_with_name(program, "omarchy-theme")
    }

    /// Activate Vencord theme with a specific theme filename
    fn activate_vencord_with_name(program: &InstalledProgram, theme_name: &str) -> Result<ActivationResult> {
        debug!("Activating Vencord theme: {}...", theme_name);

        let settings_file = program
            .config_file
            .as_ref()
            .context("Vencord settings file not found")?;

        if !settings_file.exists() {
            // If settings file doesn't exist, create a minimal one
            warn!(
                "Settings file not found at {:?}, creating new one",
                settings_file
            );

            // Create parent directory if needed
            if let Some(parent) = settings_file.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create settings directory")?;
            }

            let default_settings = json!({
                "enabledThemes": [theme_name],
                "plugins": {}
            });

            fs::write(
                settings_file,
                serde_json::to_string_pretty(&default_settings)?,
            )
            .context("Failed to write settings file")?;

            info!("✓ Created Vencord settings and enabled theme: {}", theme_name);

            return Ok(ActivationResult {
                program: program.name.clone(),
                success: true,
                message: format!("Theme enabled (created new settings): {}", theme_name),
            });
        }

        // Read existing settings
        let content = fs::read_to_string(settings_file)
            .context("Failed to read Vencord settings")?;

        let mut settings: Value = serde_json::from_str(&content)
            .context("Failed to parse Vencord settings")?;

        // Ensure enabledThemes array exists
        if !settings.get("enabledThemes").is_some() {
            settings["enabledThemes"] = json!([]);
        }

        // Get the enabled themes array
        let themes = settings["enabledThemes"]
            .as_array_mut()
            .context("enabledThemes is not an array")?;

        // Check if our theme is already enabled
        let already_enabled = themes
            .iter()
            .any(|t| t.as_str() == Some(theme_name));

        if already_enabled {
            debug!("Vencord theme {} already enabled", theme_name);
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: true,
                message: format!("Theme {} already enabled", theme_name),
            });
        }

        // Add our theme to the enabled list
        themes.push(json!(theme_name));

        // Write back to file
        fs::write(
            settings_file,
            serde_json::to_string_pretty(&settings)?,
        )
        .context("Failed to write Vencord settings")?;

        info!("✓ Enabled Vencord theme in settings: {}", theme_name);

        Ok(ActivationResult {
            program: program.name.clone(),
            success: true,
            message: format!("Theme {} enabled successfully", theme_name),
        })
    }

    /// Activate Spicetify theme using CLI
    fn activate_spicetify(program: &InstalledProgram) -> Result<ActivationResult> {
        Self::activate_spicetify_with_scheme(program, "omarchy-theme", "default")
    }

    /// Activate Spicetify with specific theme and color scheme
    fn activate_spicetify_with_scheme(
        program: &InstalledProgram,
        theme_name: &str,
        color_scheme: &str,
    ) -> Result<ActivationResult> {
        debug!("Activating Spicetify theme: {} with scheme: {}...", theme_name, color_scheme);

        if !program.cli_available {
            warn!("Spicetify CLI not available, skipping activation");
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: "CLI not available - please activate manually".to_string(),
            });
        }

        // Get the spicetify command path
        let spicetify_cmd = program.cli_path.as_ref()
            .map(|p| p.as_os_str())
            .unwrap_or_else(|| std::ffi::OsStr::new("spicetify"));

        debug!("Using spicetify command: {:?}", spicetify_cmd);

        // Set the theme
        let config_theme_output = Command::new(spicetify_cmd)
            .args(["config", "current_theme", theme_name])
            .output()
            .context("Failed to execute spicetify config current_theme")?;

        if !config_theme_output.status.success() {
            let error = String::from_utf8_lossy(&config_theme_output.stderr);
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: format!("Failed to set theme: {}", error),
            });
        }

        // Set the color scheme
        let config_scheme_output = Command::new(spicetify_cmd)
            .args(["config", "color_scheme", color_scheme])
            .output()
            .context("Failed to execute spicetify config color_scheme")?;

        if !config_scheme_output.status.success() {
            let error = String::from_utf8_lossy(&config_scheme_output.stderr);
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: format!("Failed to set color scheme: {}", error),
            });
        }

        // Apply the theme (requires Spotify restart)
        let apply_output = Command::new(spicetify_cmd)
            .arg("apply")
            .output()
            .context("Failed to execute spicetify apply command")?;

        if !apply_output.status.success() {
            let error = String::from_utf8_lossy(&apply_output.stderr);
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: format!("Failed to apply theme: {}", error),
            });
        }

        info!("✓ Activated Spicetify theme: {} with scheme: {}", theme_name, color_scheme);

        Ok(ActivationResult {
            program: program.name.clone(),
            success: true,
            message: format!("Theme {} with scheme {} applied successfully", theme_name, color_scheme),
        })
    }

    /// Deactivate theme for a program
    pub fn deactivate(program: &InstalledProgram) -> Result<ActivationResult> {
        match program.name.as_str() {
            "vencord" => Self::deactivate_vencord(program),
            "spicetify" => Self::deactivate_spicetify(program),
            _ => Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: format!("Deactivation not supported for {}", program.name),
            }),
        }
    }

    /// Deactivate Vencord theme
    fn deactivate_vencord(program: &InstalledProgram) -> Result<ActivationResult> {
        let settings_file = program
            .config_file
            .as_ref()
            .context("Vencord settings file not found")?;

        if !settings_file.exists() {
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: true,
                message: "Settings file doesn't exist".to_string(),
            });
        }

        let content = fs::read_to_string(settings_file)
            .context("Failed to read Vencord settings")?;

        let mut settings: Value = serde_json::from_str(&content)
            .context("Failed to parse Vencord settings")?;

        if let Some(themes) = settings["enabledThemes"].as_array_mut() {
            let theme_name = "omarchy-theme";
            themes.retain(|t| t.as_str() != Some(theme_name));

            fs::write(
                settings_file,
                serde_json::to_string_pretty(&settings)?,
            )
            .context("Failed to write Vencord settings")?;

            info!("✓ Disabled Vencord theme");
        }

        Ok(ActivationResult {
            program: program.name.clone(),
            success: true,
            message: "Theme disabled successfully".to_string(),
        })
    }

    /// Deactivate Spicetify theme
    fn deactivate_spicetify(program: &InstalledProgram) -> Result<ActivationResult> {
        if !program.cli_available {
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: "CLI not available".to_string(),
            });
        }

        // Get the spicetify command path
        let spicetify_cmd = program.cli_path.as_ref()
            .map(|p| p.as_os_str())
            .unwrap_or_else(|| std::ffi::OsStr::new("spicetify"));

        // Restore to default theme
        let output = Command::new(spicetify_cmd)
            .args(["config", "current_theme", ""])
            .output()
            .context("Failed to execute spicetify command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Ok(ActivationResult {
                program: program.name.clone(),
                success: false,
                message: format!("Failed to reset theme: {}", error),
            });
        }

        Command::new(spicetify_cmd)
            .arg("apply")
            .output()
            .context("Failed to apply default theme")?;

        info!("✓ Reset Spicetify to default theme");

        Ok(ActivationResult {
            program: program.name.clone(),
            success: true,
            message: "Theme disabled successfully".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_vencord_activation_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let settings_dir = temp_dir.path().join("settings");
        fs::create_dir_all(&settings_dir).unwrap();
        let settings_file = settings_dir.join("settings.json");

        let program = InstalledProgram::new(
            "vencord",
            temp_dir.path().to_path_buf(),
            Some(settings_file.clone()),
            true,
            false,
        );

        let result = ThemeActivator::activate_vencord(&program).unwrap();
        assert!(result.success);

        // Verify file was created with correct content
        let content = fs::read_to_string(&settings_file).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();
        let themes = settings["enabledThemes"].as_array().unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0], "omarchy-theme");
    }

    #[test]
    fn test_vencord_activation_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let settings_file = temp_dir.path().join("settings.json");

        // Create existing settings
        let existing = json!({
            "enabledThemes": ["other-theme.css"],
            "plugins": {"somePlugin": true}
        });
        fs::write(&settings_file, serde_json::to_string(&existing).unwrap()).unwrap();

        let program = InstalledProgram::new(
            "vencord",
            temp_dir.path().to_path_buf(),
            Some(settings_file.clone()),
            true,
            false,
        );

        let result = ThemeActivator::activate_vencord(&program).unwrap();
        assert!(result.success);

        // Verify our theme was added
        let content = fs::read_to_string(&settings_file).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();
        let themes = settings["enabledThemes"].as_array().unwrap();
        assert_eq!(themes.len(), 2);
        assert!(themes.contains(&json!("omarchy-theme")));
    }
}
