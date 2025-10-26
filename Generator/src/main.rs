mod activator;
mod color;
mod config;
mod detector;
mod extractor;
mod generator;
mod linker;
mod parser;
mod templates;
mod watcher;

use anyhow::{Context, Result};
use config::Config;
use detector::ProgramDetector;
use generator::Generator;
use linker::SymlinkManager;
use activator::ThemeActivator;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use watcher::ThemeWatcher;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mode = if args.len() > 1 {
        args[1].as_str()
    } else {
        "watch"
    };

    // Initialize logging
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|s| s.parse::<Level>().ok())
        .unwrap_or(Level::INFO);

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    // Load configuration
    let config = Config::load_or_create_default()
        .context("Failed to load configuration")?;

    info!("Omarchy Theme Generator v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from {:?}", Config::default_path());

    match mode {
        "watch" => {
            info!("Starting in watch mode...");
            run_watch_mode(&config).await
        }
        "once" | "run-once" => {
            info!("Running once for current theme...");
            run_once_mode(&config).await
        }
        "generate" => {
            if args.len() < 3 {
                eprintln!("Usage: {} generate <theme_dir>", args[0]);
                std::process::exit(1);
            }
            let theme_dir = PathBuf::from(&args[2]);
            run_generate_mode(&config, &theme_dir).await
        }
        "detect" => {
            info!("Detecting installed programs...");
            run_detect_mode()
        }
        "link" => {
            info!("Creating symlinks...");
            run_link_mode(&config)
        }
        "unlink" => {
            info!("Removing symlinks...");
            run_unlink_mode(&config)
        }
        "activate" => {
            info!("Activating themes...");
            run_activate_mode(&config)
        }
        "status" => {
            info!("Checking system status...");
            run_status_mode(&config)
        }
        "init-config" => {
            info!("Initializing default configuration...");
            init_config()
        }
        "help" | "--help" | "-h" => {
            print_help(&args[0]);
            Ok(())
        }
        _ => {
            eprintln!("Unknown mode: {}", mode);
            print_help(&args[0]);
            std::process::exit(1);
        }
    }
}

async fn run_watch_mode(config: &Config) -> Result<()> {
    let generator = Generator::new(config.clone())
        .context("Failed to create generator")?;

    let watcher = ThemeWatcher::new(config.watch_path.clone(), generator);

    info!("Watching {:?} for theme changes...", config.watch_path);
    info!("Press Ctrl+C to stop");

    watcher.watch().await
}

async fn run_once_mode(config: &Config) -> Result<()> {
    let generator = Generator::new(config.clone())
        .context("Failed to create generator")?;

    let watcher = ThemeWatcher::new(config.watch_path.clone(), generator);

    watcher.run_once().await?;

    info!("Done!");
    Ok(())
}

async fn run_generate_mode(config: &Config, theme_dir: &PathBuf) -> Result<()> {
    if !theme_dir.exists() {
        anyhow::bail!("Theme directory doesn't exist: {:?}", theme_dir);
    }

    info!("Generating theme files for {:?}", theme_dir);

    let generator = Generator::new(config.clone())
        .context("Failed to create generator")?;

    let results = generator.generate_missing_files(theme_dir)?;

    // Print results
    println!("\nGeneration Results:");
    println!("-------------------");
    for result in results {
        let status = if result.success { "✓" } else { "✗" };
        println!(
            "{} [{}] {} - {:?}",
            status, result.program, result.message, result.output_file
        );
    }

    Ok(())
}

fn run_detect_mode() -> Result<()> {
    println!("Detecting installed programs...\n");

    let programs = ProgramDetector::detect_all();

    if programs.is_empty() {
        println!("No supported programs detected.");
        println!("\nSupported programs:");
        println!("  - Vencord (Discord client mod)");
        println!("  - Spicetify (Spotify theming)");
        return Ok(());
    }

    println!("Detected Programs:");
    println!("─────────────────────────────────────────");

    for program in programs {
        println!("✓ {}", program.name.to_uppercase());
        println!("  Theme directory: {:?}", program.theme_dir);
        if let Some(config) = &program.config_file {
            println!("  Config file: {:?}", config);
        }
        if program.cli_available {
            println!("  CLI: Available");
        }
        println!();
    }

    Ok(())
}

fn run_link_mode(config: &Config) -> Result<()> {
    let installed = ProgramDetector::get_installed_enabled(&config.programs);

    if installed.is_empty() {
        println!("No installed programs detected.");
        return Ok(());
    }

    let linker = SymlinkManager::new(
        config.generated_themes_dir.clone(),
        config.create_backups,
    )?;

    println!("Creating symlinks...\n");

    for program in &installed {
        let source_file = if program.name == "spicetify" {
            "spicetify-omarchy"
        } else if program.name == "vencord" {
            "vencord.theme.css"
        } else {
            continue;
        };

        match linker.create_symlink(program, source_file) {
            Ok(result) => {
                let status = if result.success { "✓" } else { "✗" };
                println!("{} {}: {}", status, result.program, result.message);
            }
            Err(e) => {
                println!("✗ {}: Error - {:#}", program.name, e);
            }
        }
    }

    Ok(())
}

fn run_unlink_mode(config: &Config) -> Result<()> {
    let installed = ProgramDetector::get_installed_enabled(&config.programs);

    if installed.is_empty() {
        println!("No installed programs detected.");
        return Ok(());
    }

    let linker = SymlinkManager::new(
        config.generated_themes_dir.clone(),
        config.create_backups,
    )?;

    println!("Removing symlinks...\n");

    for program in &installed {
        match linker.remove_symlink(program) {
            Ok(result) => {
                let status = if result.success { "✓" } else { "✗" };
                println!("{} {}: {}", status, result.program, result.message);
            }
            Err(e) => {
                println!("✗ {}: Error - {:#}", program.name, e);
            }
        }
    }

    Ok(())
}

fn run_activate_mode(config: &Config) -> Result<()> {
    let installed = ProgramDetector::get_installed_enabled(&config.programs);

    if installed.is_empty() {
        println!("No installed programs detected.");
        return Ok(());
    }

    println!("Activating themes...\n");

    let results = ThemeActivator::activate_all(&installed);

    for result in results {
        let status = if result.success { "✓" } else { "✗" };
        println!("{} {}: {}", status, result.program, result.message);
    }

    Ok(())
}

fn run_status_mode(config: &Config) -> Result<()> {
    println!("Omarchy Theme Generator - System Status\n");

    // Check watch path
    println!("Watch Path:");
    println!("  Path: {:?}", config.watch_path);
    if config.watch_path.exists() {
        if config.watch_path.is_symlink() {
            match std::fs::read_link(&config.watch_path) {
                Ok(target) => println!("  Status: ✓ Valid symlink → {:?}", target),
                Err(_) => println!("  Status: ✗ Broken symlink"),
            }
        } else {
            println!("  Status: ✗ Not a symlink");
        }
    } else {
        println!("  Status: ✗ Does not exist");
    }
    println!();

    // Check generated themes directory
    println!("Generated Themes Directory:");
    println!("  Path: {:?}", config.generated_themes_dir);
    println!("  Exists: {}", if config.generated_themes_dir.exists() { "✓" } else { "✗" });
    println!();

    // Check configuration
    println!("Configuration:");
    println!("  Auto-activate: {}", if config.auto_activate { "✓" } else { "✗" });
    println!("  Auto-symlink: {}", if config.auto_symlink { "✓" } else { "✗" });
    println!("  Create backups: {}", if config.create_backups { "✓" } else { "✗" });
    println!();

    // Detect installed programs
    println!("Installed Programs:");
    let installed = ProgramDetector::detect_all();

    if installed.is_empty() {
        println!("  None detected");
    } else {
        for program in &installed {
            let enabled = config.programs.iter()
                .any(|p| p.name == program.name && p.enabled);

            let status = if enabled { "✓ Enabled" } else { "✗ Disabled" };
            println!("  {} - {} ({})", program.name, program.theme_dir.display(), status);
        }
    }
    println!();

    // Check enabled programs in config
    println!("Configured Programs:");
    for program in &config.programs {
        let status = if program.enabled { "✓" } else { "✗" };
        println!("  {} {} (template: {})", status, program.name, program.template);
    }

    Ok(())
}

fn init_config() -> Result<()> {
    let config_path = Config::default_path();

    if config_path.exists() {
        info!("Config file already exists at {:?}", config_path);
        return Ok(());
    }

    let config = Config::default();
    config.save(&config_path)?;

    println!("Created default configuration at: {:?}", config_path);
    println!("\nYou can edit this file to customize:");
    println!("  - Watch path (where the theme symlink is)");
    println!("  - Programs to generate configs for");
    println!("  - Color extraction priority");
    println!("  - Auto-activation and symlinking behavior");
    println!("\nDefault watch path: {:?}", config.watch_path);
    println!("Default generated themes: {:?}", config.generated_themes_dir);

    Ok(())
}

fn print_help(program_name: &str) {
    println!("Omarchy Theme Generator");
    println!("Automatically generates, symlinks, and activates themes for external programs");
    println!();
    println!("USAGE:");
    println!("    {} [MODE] [OPTIONS]", program_name);
    println!();
    println!("MODES:");
    println!("    watch           Watch for theme changes and auto-generate (default)");
    println!("    once            Generate files for current theme and exit");
    println!("    generate <dir>  Generate files for a specific theme directory");
    println!("    detect          Detect installed supported programs");
    println!("    link            Create symlinks to theme directories");
    println!("    unlink          Remove symlinks from theme directories");
    println!("    activate        Activate themes in supported programs");
    println!("    status          Show system and configuration status");
    println!("    init-config     Create default configuration file");
    println!("    help            Show this help message");
    println!();
    println!("ENVIRONMENT:");
    println!("    RUST_LOG        Set log level (trace, debug, info, warn, error)");
    println!();
    println!("EXAMPLES:");
    println!("    {}                    # Start watching for theme changes", program_name);
    println!("    {} detect              # Check which programs are installed", program_name);
    println!("    {} once                # Generate for current theme", program_name);
    println!("    {} status              # Show system status", program_name);
    println!("    {} generate ~/.config/omarchy/themes/catppuccin", program_name);
    println!("    RUST_LOG=debug {}     # Run with debug logging", program_name);
    println!();
    println!("SUPPORTED PROGRAMS:");
    println!("    - Vencord (Discord client mod)");
    println!("    - Spicetify (Spotify theming)");
    println!();
    println!("CONFIG:");
    println!("    Configuration file: {:?}", Config::default_path());
    println!("    Generated themes: ~/.config/omarchy-themes/generated/");
}
