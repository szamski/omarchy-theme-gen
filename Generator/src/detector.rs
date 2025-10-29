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
    #[allow(dead_code)]
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

        if let Some(cava) = Self::detect_cava() {
            programs.push(cava);
        }

        if let Some(tclock) = Self::detect_tclock() {
            programs.push(tclock);
        }

        if let Some(vscode) = Self::detect_vscode() {
            programs.push(vscode);
        }

        programs
    }

    /// Detect programs by config name (omarcord -> vencord, omarchify -> spicetify, omarcava -> cava)
    pub fn detect_by_config_name(config_name: &str) -> Option<InstalledProgram> {
        match config_name {
            "omarcord" => Self::detect_vencord(),
            "omarchify" => Self::detect_spicetify(),
            "omarcava" => Self::detect_cava(),
            "omarclock" => Self::detect_tclock(),
            "omarvscode" => Self::detect_vscode(),
            "vencord" => Self::detect_vencord(),
            "spicetify" => Self::detect_spicetify(),
            "cava" => Self::detect_cava(),
            "tclock" => Self::detect_tclock(),
            "vscode" => Self::detect_vscode(),
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

    /// Detect Cava installation
    pub fn detect_cava() -> Option<InstalledProgram> {
        debug!("Detecting Cava installation...");

        let home = dirs::home_dir()?;

        // Check if cava binary is available (Cava uses -v, not --version)
        let cli_available = match Command::new("cava").arg("-v").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !cli_available {
            debug!("Cava binary not found in PATH");
            return None;
        }

        // Cava config directory
        let config_dir = home.join(".config/cava");
        let config_file = config_dir.join("config");

        // We consider it "installed" if the binary is available
        // Config directory will be created during deployment if needed
        info!("✓ Cava detected (binary available in PATH)");

        Some(
            InstalledProgram::new(
                "cava",
                config_dir,
                Some(config_file),
                true,
                true, // CLI available (we just checked)
            )
            .with_cli_path(Some(PathBuf::from("cava"))),
        )
    }

    /// Detect tclock installation
    pub fn detect_tclock() -> Option<InstalledProgram> {
        debug!("Detecting tclock installation...");

        let home = dirs::home_dir()?;

        // Check if tclock binary is available
        let cli_available = match Command::new("tclock").arg("--help").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !cli_available {
            debug!("tclock binary not found in PATH");
            return None;
        }

        // tclock uses wrapper script at ~/.local/bin/omarclock
        let install_dir = home.join(".local/bin");
        let wrapper_file = install_dir.join("omarclock");

        info!("✓ tclock detected (binary available in PATH)");

        Some(
            InstalledProgram::new(
                "tclock",
                install_dir.clone(),
                Some(wrapper_file),
                true,
                true, // CLI available (we just checked)
            )
            .with_cli_path(Some(PathBuf::from("tclock"))),
        )
    }

    /// Detect VS Code installation
    pub fn detect_vscode() -> Option<InstalledProgram> {
        debug!("Detecting VS Code installation...");

        let home = dirs::home_dir()?;

        // Check for VS Code extensions directory
        let vscode_ext_dir = home.join(".vscode/extensions");

        if !vscode_ext_dir.exists() {
            debug!("VS Code extensions directory not found");
            return None;
        }

        // VS Code theme will be installed at ~/.vscode/extensions/local.theme-omarvscode/
        // (directory name follows publisher.name convention)
        let theme_dir = vscode_ext_dir.join("local.theme-omarvscode");
        let theme_file = theme_dir.join("themes/omarvscode-color-theme.json");

        // Check if 'code' command is available
        let cli_available = match Command::new("code").arg("--version").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        info!("✓ VS Code detected (extensions directory exists)");

        Some(
            InstalledProgram::new(
                "vscode",
                theme_dir,
                Some(theme_file),
                true,
                cli_available,
            )
            .with_cli_path(if cli_available {
                Some(PathBuf::from("code"))
            } else {
                None
            }),
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
