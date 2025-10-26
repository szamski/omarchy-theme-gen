use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

/// Represents an installed program that can be themed
#[derive(Debug, Clone)]
pub struct InstalledProgram {
    /// Name of the program (e.g., "vencord", "spicetify")
    pub name: String,

    /// Directory where themes should be placed
    pub theme_dir: PathBuf,

    /// Optional configuration file path
    pub config_file: Option<PathBuf>,

    /// Whether the program is installed
    pub is_installed: bool,

    /// Whether CLI tool is available (for programs that need it)
    pub cli_available: bool,

    /// Full path to CLI executable (if available)
    pub cli_path: Option<PathBuf>,
}

impl InstalledProgram {
    /// Create a new InstalledProgram instance
    pub fn new(
        name: impl Into<String>,
        theme_dir: PathBuf,
        config_file: Option<PathBuf>,
        is_installed: bool,
        cli_available: bool,
    ) -> Self {
        Self {
            name: name.into(),
            theme_dir,
            config_file,
            is_installed,
            cli_available,
            cli_path: None,
        }
    }

    /// Create a new InstalledProgram instance with CLI path
    pub fn with_cli_path(mut self, cli_path: Option<PathBuf>) -> Self {
        self.cli_path = cli_path;
        self
    }
}

/// Detects installed programs that can be themed
pub struct ProgramDetector;

impl ProgramDetector {
    /// Detect all supported programs
    pub fn detect_all() -> Vec<InstalledProgram> {
        let mut programs = Vec::new();

        if let Some(vencord) = Self::detect_vencord() {
            programs.push(vencord);
        }

        if let Some(spicetify) = Self::detect_spicetify() {
            programs.push(spicetify);
        }

        programs
    }

    /// Detect programs by config name (omarcord -> vencord, omarchify -> spicetify)
    pub fn detect_by_config_name(config_name: &str) -> Option<InstalledProgram> {
        match config_name {
            "omarcord" => Self::detect_vencord(),
            "omarchify" => Self::detect_spicetify(),
            "vencord" => Self::detect_vencord(),
            "spicetify" => Self::detect_spicetify(),
            _ => None,
        }
    }

    /// Detect Vencord installation
    pub fn detect_vencord() -> Option<InstalledProgram> {
        debug!("Detecting Vencord installation...");

        let home = dirs::home_dir()?;

        // Check multiple possible locations
        let possible_paths = vec![
            // Standard installation
            home.join(".config/Vencord/themes"),
            // Flatpak Vesktop
            home.join(".var/app/dev.vencord.Vesktop/config/Vencord/themes"),
        ];

        for theme_dir in possible_paths {
            if theme_dir.exists() && theme_dir.is_dir() {
                let config_file = theme_dir
                    .parent()
                    .map(|p| p.join("settings/settings.json"));

                info!("✓ Vencord detected at: {:?}", theme_dir);

                return Some(InstalledProgram::new(
                    "vencord",
                    theme_dir,
                    config_file,
                    true,
                    false, // Vencord doesn't need CLI
                ));
            }
        }

        debug!("Vencord not detected");
        None
    }

    /// Detect Spicetify installation
    pub fn detect_spicetify() -> Option<InstalledProgram> {
        debug!("Detecting Spicetify installation...");

        let home = dirs::home_dir()?;

        // Check multiple possible locations
        let possible_paths = vec![
            // Standard installation
            home.join(".config/spicetify/Themes"),
            // Flatpak Spotify
            home.join(".var/app/com.spotify.Client/config/spicetify/Themes"),
        ];

        let mut theme_dir = None;

        for path in possible_paths {
            if path.exists() && path.is_dir() {
                theme_dir = Some(path);
                break;
            }
        }

        let theme_dir = theme_dir?;

        // Check if CLI is available and find its path
        let mut cli_path: Option<PathBuf> = None;
        let mut cli_available = Self::check_cli_available("spicetify");

        if !cli_available {
            // Try common installation paths
            let spicetify_paths = vec![
                home.join(".spicetify/spicetify"),
                home.join(".local/bin/spicetify"),
                PathBuf::from("/usr/local/bin/spicetify"),
            ];

            for path in spicetify_paths {
                if path.exists() {
                    debug!("Found spicetify at: {:?}", path);
                    // Test if it works
                    match Command::new(&path).arg("--version").output() {
                        Ok(output) if output.status.success() => {
                            cli_available = true;
                            cli_path = Some(path.clone());
                            info!("✓ Spicetify CLI found at: {:?}", path);
                            break;
                        }
                        _ => continue,
                    }
                }
            }
        } else {
            // spicetify is in PATH, just use "spicetify" as the command
            cli_path = Some(PathBuf::from("spicetify"));
        }

        if cli_available {
            info!("✓ Spicetify detected at: {:?} (CLI available)", theme_dir);
        } else {
            info!(
                "✓ Spicetify detected at: {:?} (CLI not available - activation disabled)",
                theme_dir
            );
        }

        Some(
            InstalledProgram::new("spicetify", theme_dir, None, true, cli_available)
                .with_cli_path(cli_path),
        )
    }

    /// Check if a CLI command is available
    pub fn check_cli_available(program: &str) -> bool {
        match Command::new(program).arg("--version").output() {
            Ok(output) => {
                let success = output.status.success();
                debug!(
                    "{} CLI check: {}",
                    program,
                    if success { "available" } else { "not available" }
                );
                success
            }
            Err(_) => {
                debug!("{} CLI not found", program);
                false
            }
        }
    }

    /// Get installed programs filtered by configuration
    pub fn get_installed_enabled(
        config_programs: &[crate::config::ProgramConfig],
    ) -> Vec<InstalledProgram> {
        let detected = Self::detect_all();

        detected
            .into_iter()
            .filter(|installed| {
                // Check if program is enabled in config
                config_programs
                    .iter()
                    .any(|cfg| cfg.name == installed.name && cfg.enabled)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_all() {
        let programs = ProgramDetector::detect_all();
        // This will vary by system, just ensure it doesn't panic
        println!("Detected programs: {:#?}", programs);
    }

    #[test]
    fn test_cli_check() {
        // Test with a command that should always exist
        let result = ProgramDetector::check_cli_available("ls");
        assert!(result);

        // Test with a command that probably doesn't exist
        let result = ProgramDetector::check_cli_available("nonexistent-command-12345");
        assert!(!result);
    }
}
