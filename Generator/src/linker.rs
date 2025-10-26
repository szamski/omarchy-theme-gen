use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::detector::InstalledProgram;

/// Manages symlinks for theme files
pub struct SymlinkManager {
    /// Source directory where generated themes are stored
    source_dir: PathBuf,

    /// Directory for backing up conflicting files
    backup_dir: PathBuf,

    /// Whether to create backups before replacing files
    create_backups: bool,
}

/// Result of a symlink operation
#[derive(Debug)]
pub struct LinkResult {
    pub program: String,
    pub target_path: PathBuf,
    pub success: bool,
    pub message: String,
}

impl SymlinkManager {
    /// Create a new SymlinkManager
    pub fn new(source_dir: PathBuf, create_backups: bool) -> Result<Self> {
        let backup_dir = source_dir
            .parent()
            .context("Invalid source directory")?
            .join("backups");

        // Create source directory if it doesn't exist
        if !source_dir.exists() {
            fs::create_dir_all(&source_dir)
                .with_context(|| format!("Failed to create source directory: {:?}", source_dir))?;
        }

        // Create backup directory if needed
        if create_backups && !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .with_context(|| format!("Failed to create backup directory: {:?}", backup_dir))?;
        }

        Ok(Self {
            source_dir,
            backup_dir,
            create_backups,
        })
    }

    /// Get the source directory for generated themes
    pub fn get_source_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/omarchy-themes/generated")
    }

    /// Create symlink for a program
    pub fn create_symlink(
        &self,
        program: &InstalledProgram,
        source_filename: &str,
    ) -> Result<LinkResult> {
        let source_path = self.source_dir.join(source_filename);
        let target_path = program.theme_dir.join("omarchy-theme");

        debug!(
            "Creating symlink for {}: {:?} -> {:?}",
            program.name, target_path, source_path
        );

        // Ensure source file/directory exists
        if !source_path.exists() {
            return Ok(LinkResult {
                program: program.name.clone(),
                target_path: target_path.clone(),
                success: false,
                message: format!("Source file not found: {:?}", source_path),
            });
        }

        // Check if target already exists
        if target_path.exists() || target_path.is_symlink() {
            if let Ok(link_target) = fs::read_link(&target_path) {
                // It's a symlink
                if link_target == source_path {
                    // Already points to the correct location
                    debug!("Symlink already exists and is correct");
                    return Ok(LinkResult {
                        program: program.name.clone(),
                        target_path: target_path.clone(),
                        success: true,
                        message: "Symlink already exists".to_string(),
                    });
                } else {
                    // Points to wrong location, remove and recreate
                    warn!("Removing incorrect symlink: {:?}", target_path);
                    fs::remove_file(&target_path)
                        .context("Failed to remove old symlink")?;
                }
            } else {
                // It's a regular file/directory, backup if needed
                if self.create_backups {
                    self.backup_existing(&target_path)
                        .context("Failed to backup existing file")?;
                }

                // Remove the existing file/directory
                if target_path.is_dir() {
                    fs::remove_dir_all(&target_path)
                        .context("Failed to remove existing directory")?;
                } else {
                    fs::remove_file(&target_path)
                        .context("Failed to remove existing file")?;
                }
            }
        }

        // Create the symlink
        unix_fs::symlink(&source_path, &target_path).with_context(|| {
            format!(
                "Failed to create symlink: {:?} -> {:?}",
                target_path, source_path
            )
        })?;

        info!("✓ Created symlink: {:?} -> {:?}", target_path, source_path);

        Ok(LinkResult {
            program: program.name.clone(),
            target_path,
            success: true,
            message: "Symlink created successfully".to_string(),
        })
    }

    /// Backup an existing file or directory
    fn backup_existing(&self, path: &Path) -> Result<()> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = path
            .file_name()
            .context("Invalid path")?
            .to_string_lossy();
        let backup_name = format!("{}_{}", filename, timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        if path.is_dir() {
            fs_extra::dir::copy(
                path,
                &backup_path,
                &fs_extra::dir::CopyOptions::new(),
            )
            .context("Failed to backup directory")?;
        } else {
            fs::copy(path, &backup_path)
                .context("Failed to backup file")?;
        }

        info!("Backed up existing file to: {:?}", backup_path);
        Ok(())
    }

    /// Remove symlink for a program
    pub fn remove_symlink(&self, program: &InstalledProgram) -> Result<LinkResult> {
        let target_path = program.theme_dir.join("omarchy-theme");

        if !target_path.exists() && !target_path.is_symlink() {
            return Ok(LinkResult {
                program: program.name.clone(),
                target_path: target_path.clone(),
                success: true,
                message: "Symlink doesn't exist".to_string(),
            });
        }

        // Only remove if it's a symlink
        if target_path.is_symlink() {
            fs::remove_file(&target_path)
                .context("Failed to remove symlink")?;

            info!("Removed symlink: {:?}", target_path);

            Ok(LinkResult {
                program: program.name.clone(),
                target_path,
                success: true,
                message: "Symlink removed successfully".to_string(),
            })
        } else {
            Ok(LinkResult {
                program: program.name.clone(),
                target_path,
                success: false,
                message: "Not a symlink, skipping removal".to_string(),
            })
        }
    }

    /// Clean up broken symlinks in a directory
    pub fn cleanup_broken_links(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut cleaned = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(cleaned);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_symlink() {
                // Check if the symlink is broken
                if !path.exists() {
                    debug!("Removing broken symlink: {:?}", path);
                    fs::remove_file(&path)?;
                    cleaned.push(path);
                }
            }
        }

        if !cleaned.is_empty() {
            info!("Cleaned up {} broken symlinks", cleaned.len());
        }

        Ok(cleaned)
    }

    /// Verify that a symlink points to the expected location
    pub fn verify_link(&self, path: &Path, expected_target: &Path) -> bool {
        if !path.is_symlink() {
            return false;
        }

        match fs::read_link(path) {
            Ok(target) => target == expected_target,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_symlink_creation() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("generated");
        let target_dir = temp_dir.path().join("themes");

        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        // Create a source file
        let source_file = source_dir.join("test.css");
        fs::write(&source_file, "test content").unwrap();

        let manager = SymlinkManager::new(source_dir.clone(), true).unwrap();

        let program = InstalledProgram::new(
            "test",
            target_dir.clone(),
            None,
            true,
            false,
        );

        let result = manager.create_symlink(&program, "test.css").unwrap();
        assert!(result.success);

        let link_path = target_dir.join("omarchy-theme");
        assert!(link_path.is_symlink());
        assert_eq!(fs::read_link(&link_path).unwrap(), source_file);
    }

    #[test]
    fn test_verify_link() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let link = temp_dir.path().join("link.txt");

        fs::write(&source, "content").unwrap();
        unix_fs::symlink(&source, &link).unwrap();

        let manager = SymlinkManager::new(temp_dir.path().to_path_buf(), false).unwrap();

        assert!(manager.verify_link(&link, &source));
        assert!(!manager.verify_link(&link, &temp_dir.path().join("other.txt")));
    }
}
