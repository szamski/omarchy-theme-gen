use crate::generator::Generator;
use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Watch for theme changes and generate missing files
pub struct ThemeWatcher {
    watch_path: PathBuf,
    generator: Generator,
}

impl ThemeWatcher {
    /// Create a new theme watcher
    pub fn new(watch_path: PathBuf, generator: Generator) -> Self {
        ThemeWatcher {
            watch_path,
            generator,
        }
    }

    /// Start watching for theme changes
    pub async fn watch(&self) -> Result<()> {
        info!("Starting theme watcher on {:?}", self.watch_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.watch_path.parent() {
            if !parent.exists() {
                warn!(
                    "Watch path parent directory doesn't exist: {:?}. Waiting for it to be created...",
                    parent
                );
                // Could wait for directory to be created, but for now just continue
            }
        }

        // Run initial generation if symlink exists
        if self.watch_path.exists() {
            if let Err(e) = self.handle_theme_change().await {
                error!("Error during initial theme file generation: {}", e);
            }
        } else {
            warn!("Watch path doesn't exist yet: {:?}", self.watch_path);
        }

        // Set up file watcher
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if let Err(e) = tx.send(event) {
                        error!("Failed to send event: {}", e);
                    }
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_secs(2)),
        )?;

        // Watch parent directory since we're watching a symlink
        let watch_dir = if let Some(parent) = self.watch_path.parent() {
            parent
        } else {
            &self.watch_path
        };

        watcher
            .watch(watch_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("Failed to watch directory: {:?}", watch_dir))?;

        info!("Watching for changes in {:?}", watch_dir);

        // Process events
        loop {
            match rx.recv() {
                Ok(event) => {
                    if self.is_relevant_event(&event) {
                        info!("Theme change detected: {:?}", event.kind);

                        // Wait a bit for the symlink to settle
                        sleep(Duration::from_millis(500)).await;

                        if let Err(e) = self.handle_theme_change().await {
                            error!("Error handling theme change: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Watch error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check if an event is relevant to our watch path
    fn is_relevant_event(&self, event: &Event) -> bool {
        // Check if any of the paths in the event match our watch path
        for path in &event.paths {
            if path == &self.watch_path || path.parent() == self.watch_path.parent() {
                // We care about create, modify, and remove events
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        return true;
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Handle a theme change event
    async fn handle_theme_change(&self) -> Result<()> {
        // Read the symlink to get the target theme directory
        let theme_dir = fs::read_link(&self.watch_path)
            .with_context(|| format!("Failed to read symlink: {:?}", self.watch_path))?;

        // If it's a relative path, make it absolute relative to the symlink's parent
        let theme_dir = if theme_dir.is_relative() {
            if let Some(parent) = self.watch_path.parent() {
                parent.join(theme_dir)
            } else {
                theme_dir
            }
        } else {
            theme_dir
        };

        info!("Theme changed to: {:?}", theme_dir);

        // Verify theme directory exists
        if !theme_dir.exists() {
            warn!("Theme directory doesn't exist: {:?}", theme_dir);
            return Ok(());
        }

        // Use the new generate_and_deploy method for full workflow
        self.generator.generate_and_deploy(&theme_dir)?;

        Ok(())
    }

    /// Run once (generate files for current theme and exit)
    pub async fn run_once(&self) -> Result<()> {
        info!("Running once for current theme at {:?}", self.watch_path);

        if !self.watch_path.exists() {
            anyhow::bail!("Watch path doesn't exist: {:?}", self.watch_path);
        }

        self.handle_theme_change().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_theme_watcher_creation() {
        let config = Config::default();
        let generator = Generator::new(config).unwrap();
        let watcher = ThemeWatcher::new(PathBuf::from("/tmp/test"), generator);

        assert_eq!(watcher.watch_path, PathBuf::from("/tmp/test"));
    }

    #[tokio::test]
    async fn test_run_once() {
        let temp_dir = TempDir::new().unwrap();
        let theme_dir = temp_dir.path().join("theme");
        fs::create_dir(&theme_dir).unwrap();

        // Create alacritty.toml in theme
        fs::write(
            theme_dir.join("alacritty.toml"),
            "[colors.primary]\nbackground = \"#eff1f5\"\nforeground = \"#4c4f69\"\n",
        )
        .unwrap();

        // Create symlink
        let symlink_path = temp_dir.path().join("current");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&theme_dir, &symlink_path).unwrap();

        let config = Config::default();
        let generator = Generator::new(config).unwrap();
        let watcher = ThemeWatcher::new(symlink_path, generator);

        let result = watcher.run_once().await;
        assert!(result.is_ok());
    }
}
