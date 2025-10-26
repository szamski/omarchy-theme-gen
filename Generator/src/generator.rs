use crate::activator::ThemeActivator;
use crate::color::ColorPalette;
use crate::config::{Config, ProgramConfig};
use crate::detector::ProgramDetector;
use crate::extractor::{self, ColorSource};
use crate::linker::SymlinkManager;
use crate::templates::TemplateRenderer;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Result of a generation operation
#[derive(Debug)]
pub struct GenerationResult {
    pub program: String,
    pub output_file: PathBuf,
    pub success: bool,
    pub message: String,
}

/// Theme file generator
pub struct Generator {
    renderer: TemplateRenderer,
    config: Config,
}

impl Generator {
    /// Create a new generator with the given config
    pub fn new(config: Config) -> Result<Self> {
        let renderer = TemplateRenderer::new(None)?;
        Ok(Generator { renderer, config })
    }

    /// Generate missing theme files for a theme directory
    pub fn generate_missing_files(&self, theme_dir: &Path) -> Result<Vec<GenerationResult>> {
        info!("Generating missing files for theme: {:?}", theme_dir);

        // Extract color palette from theme directory
        let color_priority = self.parse_color_priority();
        let (palette, source) = extractor::extract_colors(theme_dir, &color_priority)
            .with_context(|| format!("Failed to extract colors from {:?}", theme_dir))?;

        info!("Extracted colors from {:?}", source.filename());

        let mut results = Vec::new();

        // Check each enabled program
        for program in self.config.enabled_programs() {
            let result = self.generate_for_program(theme_dir, &palette, program);
            results.push(result);
        }

        Ok(results)
    }

    /// Full workflow: detect programs, generate themes, create symlinks, and activate
    pub fn generate_and_deploy(&self, theme_dir: &Path) -> Result<()> {
        info!("Starting full theme deployment workflow...");

        // Extract color palette from theme directory
        let color_priority = self.parse_color_priority();
        let (palette, source) = extractor::extract_colors(theme_dir, &color_priority)
            .with_context(|| format!("Failed to extract colors from {:?}", theme_dir))?;

        info!("✓ Extracted colors from {:?}", source.filename());

        // Process each enabled program
        for program_config in self.config.enabled_programs() {
            // Detect if this program is installed
            let installed = ProgramDetector::detect_by_config_name(&program_config.name);

            if installed.is_none() {
                info!("⊘ {} not installed, skipping", program_config.name);
                continue;
            }

            let installed = installed.unwrap();
            info!("✓ Detected {}", program_config.name);

            // Handle each program type differently
            match program_config.name.as_str() {
                "omarcord" => self.deploy_omarcord(theme_dir, &palette, program_config, &installed)?,
                "omarchify" => self.deploy_omarchify(theme_dir, &palette, program_config, &installed)?,
                _ => warn!("Unknown program type: {}", program_config.name),
            }
        }

        info!("✓ Theme deployment complete!");
        Ok(())
    }

    /// Deploy Omarcord (Discord theme)
    fn deploy_omarcord(
        &self,
        _theme_dir: &Path,
        palette: &ColorPalette,
        program_config: &ProgramConfig,
        installed: &crate::detector::InstalledProgram,
    ) -> Result<()> {
        // 1. Render the full omarcord.theme.css template
        let content = self.renderer.render(&program_config.template, palette, &program_config.variables)
            .context("Failed to render Omarcord template")?;

        // 2. Save to centralized location (for backup/reference)
        let output_dir = &self.config.generated_themes_dir;
        fs::create_dir_all(output_dir)?;
        let generated_file = output_dir.join(&program_config.output_file);
        fs::write(&generated_file, &content)?;
        info!("✓ Generated Omarcord theme: {:?}", generated_file);

        // 3. Write directly to Vencord themes folder (Vencord doesn't support symlinks)
        let vencord_theme_file = installed.theme_dir.join(&program_config.output_file);

        // Backup existing file if it exists and backups are enabled
        if vencord_theme_file.exists() && self.config.create_backups {
            let backup_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/omarchy-themes/backups");
            fs::create_dir_all(&backup_dir)?;

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_file = backup_dir.join(format!("omarcord.theme.css.{}", timestamp));
            fs::copy(&vencord_theme_file, &backup_file).ok();
        }

        // Write the theme file directly
        fs::write(&vencord_theme_file, content)?;
        info!("✓ Wrote Omarcord theme to Vencord: {:?}", vencord_theme_file);

        // 4. Activate if enabled
        if self.config.auto_activate {
            let result = ThemeActivator::activate_omarcord(installed)?;
            if result.success {
                info!("✓ Activated Omarcord: {}", result.message);
            } else {
                warn!("✗ Activation failed: {}", result.message);
            }
        }

        Ok(())
    }

    /// Deploy Omarchify (Spotify theme)
    fn deploy_omarchify(
        &self,
        theme_dir: &Path,
        palette: &ColorPalette,
        program_config: &ProgramConfig,
        installed: &crate::detector::InstalledProgram,
    ) -> Result<()> {
        // 1. Read the base color.ini file from Omarchify repo
        let base_file_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("programming/omarchy-theme-gen/Omarchify/text/color.ini");

        let base_content = if base_file_path.exists() {
            fs::read_to_string(&base_file_path)
                .with_context(|| format!("Failed to read base color.ini from {:?}", base_file_path))?
        } else {
            warn!("Base color.ini not found at {:?}, using minimal base", base_file_path);
            String::from("; Omarchify color schemes\n\n")
        };

        // 2. Render the [Omarchify] section
        let omarchify_section = self.renderer.render("omarchify-colors", palette, &program_config.variables)
            .context("Failed to render Omarchify color section")?;

        // 3. Combine: base content + new section
        let combined = format!("{}\n{}", base_content, omarchify_section);

        // 4. Save to theme directory
        let theme_color_file = theme_dir.join(&program_config.output_file);
        fs::write(&theme_color_file, combined)?;
        info!("✓ Generated Omarchify color.ini: {:?}", theme_color_file);

        // 5. Create symlink from Spicetify Themes/text/ to watch_path's color.ini
        //    This ensures Spicetify always uses the current theme
        if self.config.auto_symlink {
            let spicetify_text_dir = installed.theme_dir.join("text");
            fs::create_dir_all(&spicetify_text_dir)?;

            let symlink_target = spicetify_text_dir.join("color.ini");

            // Symlink to watch_path (current theme) instead of specific theme_dir
            // This way, when Omarchy changes themes, Spicetify automatically picks it up
            let symlink_source = self.config.watch_path.join(&program_config.output_file);

            // Remove existing symlink or file
            if symlink_target.exists() || symlink_target.is_symlink() {
                fs::remove_file(&symlink_target).ok();
            }

            std::os::unix::fs::symlink(&symlink_source, &symlink_target)?;
            info!("✓ Symlinked Omarchify: {:?} -> {:?}", symlink_target, symlink_source);
        }

        // 6. Activate if enabled
        if self.config.auto_activate {
            let result = ThemeActivator::activate_omarchify(installed)?;
            if result.success {
                info!("✓ Activated Omarchify: {}", result.message);
            } else {
                warn!("✗ Activation failed: {}", result.message);
            }
        }

        Ok(())
    }

    /// Generate theme file for a specific program
    fn generate_for_program(
        &self,
        theme_dir: &Path,
        palette: &ColorPalette,
        program: &ProgramConfig,
    ) -> GenerationResult {
        let output_path = theme_dir.join(&program.output_file);

        // Check if file already exists
        if output_path.exists() {
            info!(
                "Skipping {} - file already exists: {:?}",
                program.name, output_path
            );
            return GenerationResult {
                program: program.name.clone(),
                output_file: output_path,
                success: true,
                message: "File already exists (skipped)".to_string(),
            };
        }

        // Render template
        match self.renderer.render(&program.template, palette, &program.variables) {
            Ok(content) => {
                // Write file
                match fs::write(&output_path, content) {
                    Ok(_) => {
                        info!(
                            "Generated {} theme file: {:?}",
                            program.name, output_path
                        );
                        GenerationResult {
                            program: program.name.clone(),
                            output_file: output_path,
                            success: true,
                            message: "Generated successfully".to_string(),
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to write {} theme file {:?}: {}",
                            program.name, output_path, e
                        );
                        GenerationResult {
                            program: program.name.clone(),
                            output_file: output_path,
                            success: false,
                            message: format!("Write error: {}", e),
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to render {} template: {}",
                    program.name, e
                );
                GenerationResult {
                    program: program.name.clone(),
                    output_file: output_path,
                    success: false,
                    message: format!("Template error: {}", e),
                }
            }
        }
    }

    /// Parse color priority from config
    fn parse_color_priority(&self) -> Vec<ColorSource> {
        self.config
            .color_priority
            .iter()
            .filter_map(|s| match s.as_str() {
                "alacritty.toml" => Some(ColorSource::Alacritty),
                "btop.theme" => Some(ColorSource::Btop),
                "custom_theme.json" => Some(ColorSource::CustomJson),
                _ => {
                    warn!("Unknown color source in config: {}", s);
                    None
                }
            })
            .collect()
    }

    /// Force regenerate all files (even if they exist)
    #[allow(dead_code)]
    pub fn regenerate_all_files(&self, theme_dir: &Path) -> Result<Vec<GenerationResult>> {
        info!("Regenerating all files for theme: {:?}", theme_dir);

        // Extract color palette
        let color_priority = self.parse_color_priority();
        let (palette, source) = extractor::extract_colors(theme_dir, &color_priority)
            .with_context(|| format!("Failed to extract colors from {:?}", theme_dir))?;

        info!("Extracted colors from {:?}", source.filename());

        let mut results = Vec::new();

        // Generate for each enabled program (delete existing files first)
        for program in self.config.enabled_programs() {
            let output_path = theme_dir.join(&program.output_file);

            // Delete if exists
            if output_path.exists() {
                if let Err(e) = fs::remove_file(&output_path) {
                    warn!("Failed to delete existing file {:?}: {}", output_path, e);
                }
            }

            let result = self.generate_for_program(theme_dir, &palette, program);
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generator() {
        let temp_dir = TempDir::new().unwrap();

        // Create a sample alacritty.toml
        let alacritty_path = temp_dir.path().join("alacritty.toml");
        fs::write(
            &alacritty_path,
            "[colors.primary]\nbackground = \"#eff1f5\"\nforeground = \"#4c4f69\"\n",
        )
        .unwrap();

        let config = Config::default();
        let generator = Generator::new(config).unwrap();

        let results = generator.generate_missing_files(temp_dir.path()).unwrap();

        // Should have results for each program
        assert!(results.len() > 0);

        // Check if files were created
        for result in results {
            if result.success {
                assert!(result.output_file.exists());
            }
        }
    }
}
