# Omarchy Theme Generator

Automatically synchronize your [Omarchy](https://github.com/szamski/omarchy) theme colors with Discord (Vencord), Spotify (Spicetify), Cava (audio visualizer), and more.

## Overview

Omarchy Theme Generator is a Rust-based CLI tool that watches for changes in your Omarchy theme and automatically generates matching color schemes for:

- **Omarcord**: A System24-based theme for Discord (via Vencord)
- **Omarchify**: A text-based theme for Spotify (via Spicetify)
- **Omarcava**: A Cyberpunk 2077-inspired theme for Cava audio visualizer
- **Omarclock**: A futuristic wrapper for tclock terminal clock
- **Omarvscode**: A VS Code theme with vibrant yellow/red status bar

When you change your Omarchy theme, all enabled applications update automatically to match, including the signature turquoise accents.

## Features

- **Automatic Color Synchronization**: Extracts colors from your Omarchy theme files (alacritty.toml, btop.theme, or custom_theme.json)
- **Real-time Watching**: Monitors theme changes and updates instantly
- **One-Click Installation**: Comprehensive install script handles everything
- **Auto-Activation**: Themes are automatically activated in Vencord and Spicetify
- **Smart Deployment**:
  - Omarcord: Full theme generation with color injection (instant updates via Vencord)
  - Omarchify: Color section appended to base theme (requires Spotify restart)
- **Systemd Integration**: Optional auto-start on login
- **CLI Path Detection**: Finds Spicetify even in non-standard locations

## Requirements

- **Rust**: For building the generator (installed automatically by install.sh)
- **Omarchy**: Your base theme system
- **Discord/Vesktop** (optional): For Discord theming
- **Vencord** (optional): Discord client mod - installed via install.sh
- **Spotify** (optional): For Spotify theming
- **Spicetify** (optional): Spotify theming CLI - installed via install.sh
- **Cava** (optional): Audio visualizer - `sudo pacman -S cava` (Arch) or `sudo apt install cava` (Debian/Ubuntu)
- **tclock** (optional): Terminal clock - `cargo install tclock` or install via [tclock](https://github.com/nwrenger/tclock)

## Installation

### Quick Install

```bash
git clone https://github.com/yourusername/omarchy-theme-gen.git
cd omarchy-theme-gen
./install.sh
```

The install script will:
1. Build and install the theme generator binary
2. Initialize configuration
3. Optionally install Spicetify + text theme
4. Optionally install Vencord (guides you through web installer)
5. Set up systemd service for auto-start
6. Generate initial themes

### Manual Installation

If you prefer manual installation:

```bash
# Build the generator
cd Generator
cargo build --release

# Install binary
cp target/release/omarchy-theme-gen ~/.local/bin/

# Initialize config
omarchy-theme-gen init-config

# Edit config if needed
nano ~/.config/omarchy-theme-watcher/config.toml
```

### Reinstall / Uninstall

To update or reinstall the generator:

```bash
# Reinstall (uninstall + fresh install)
./reinstall.sh

# Uninstall only
./uninstall.sh
```

The reinstall script will:
- Uninstall the current version (keeping config backup)
- Clean build artifacts
- Build and install fresh version
- Preserve Spicetify and Vencord installations

## Usage

### Command Line

```bash
# Generate themes once for current Omarchy theme
omarchy-theme-gen once

# Watch for Omarchy theme changes (runs continuously)
omarchy-theme-gen watch

# Detect installed programs
omarchy-theme-gen detect

# Show system status
omarchy-theme-gen status

# Initialize/reset configuration
omarchy-theme-gen init-config

# Show help
omarchy-theme-gen help
```

### Systemd Service (Recommended)

The systemd service automatically starts the watcher on login:

```bash
# Check service status
systemctl --user status omarchy-theme-gen

# Start/stop service
systemctl --user start omarchy-theme-gen
systemctl --user stop omarchy-theme-gen

# Enable/disable auto-start
systemctl --user enable omarchy-theme-gen
systemctl --user disable omarchy-theme-gen

# View logs
journalctl --user -u omarchy-theme-gen -f
```

## Configuration

Configuration file: `~/.config/omarchy-theme-watcher/config.toml`

```toml
# Path to current Omarchy theme directory
watch_path = "/home/yourusername/.config/omarchy/current/theme"

# Where to store generated theme files
generated_themes_dir = "/home/yourusername/.config/omarchy-themes/generated"

# Color extraction priority (first found wins)
color_priority = ["alacritty.toml", "btop.theme", "custom_theme.json"]

# Enabled programs
[[programs]]
name = "omarcord"
enabled = true
output_file = "omarcord.theme.css"
template = "omarcord"

[[programs]]
name = "omarchify"
enabled = true
output_file = "color.ini"
template = "omarchify"

[[programs]]
name = "omarcava"
enabled = true
output_file = "config"
template = "omarcava"

[[programs]]
name = "omarclock"
enabled = true
output_file = "omarclock.sh"
template = "omarclock"

[[programs]]
name = "omarvscode"
enabled = true
output_file = "omarvscode-color-theme.json"
template = "omarvscode"

# Options
auto_symlink = true       # Create symlinks to theme directories
auto_activate = true      # Automatically activate themes
create_backups = true     # Backup existing theme files
```

## Project Structure

```
omarchy-theme-gen/
├── Generator/              # Rust theme generator CLI
│   ├── src/
│   │   ├── main.rs        # CLI entry point
│   │   ├── generator.rs   # Theme generation logic
│   │   ├── detector.rs    # Program installation detection
│   │   ├── activator.rs   # Theme activation
│   │   ├── extractor.rs   # Color extraction from Omarchy
│   │   ├── watcher.rs     # File system watching
│   │   └── ...
│   └── templates/
│       ├── omarcord.theme.css      # Full Discord theme template
│       ├── omarchify-colors.ini    # Spotify color section
│       ├── omarcava.config         # Cava audio visualizer config
│       └── omarclock.sh            # tclock wrapper script
│
├── Omarcord/              # System24 fork for Discord
│   ├── assets/            # Theme assets (fonts, images)
│   └── theme/
│       └── omarcord.theme.css
│
├── Omarchify/             # Text theme fork for Spotify
│   └── text/
│       ├── color.ini      # Base color schemes
│       └── user.css       # Theme styles
│
├── install.sh             # Comprehensive installation script
├── reinstall.sh           # Quick reinstall script
├── uninstall.sh           # Uninstallation script
├── LICENSE                # MIT License
└── README.md              # This file
```

## How It Works

### Omarcord (Discord Theme)

1. **Template**: Uses full System24 theme structure (`omarcord.theme.css`)
2. **Color Injection**: Replaces color variables with Omarchy theme colors
3. **Deployment**: Writes complete theme file directly to Vencord themes folder
4. **Activation**: Updates Vencord settings.json to enable the theme
5. **Updates**: Instant - Discord picks up changes immediately

### Omarchify (Spotify Theme)

1. **Base Theme**: Reads the original text theme's `color.ini` (with all color schemes)
2. **Section Generation**: Creates new `[Omarchify]` section with current colors
3. **Combination**: Appends Omarchify section to base file
4. **Deployment**: Saves to theme directory (symlinked to current theme)
5. **Activation**: Runs `spicetify config color_scheme Omarchify && spicetify apply`
6. **Updates**: Requires Spotify restart

### Omarcava (Cava Audio Visualizer)

**Cyberpunk 2077-Inspired Aesthetic**

1. **Template**: Full Cava config with 8-color vertical gradient
2. **Color Mapping**:
   - Gradient: purple → magenta → pink → cyan → turquoise → green → white
   - Maps to theme colors: `bright_magenta`, `magenta`, `bright_red`, `bright_cyan`, `cyan`, `bright_green`, `green`, `foreground`
3. **Deployment**: Writes directly to `~/.config/cava/config`
4. **Backup**: Creates timestamped backup if config exists
5. **Activation**: Sends notification to reload (Cava requires pressing 'r' or restart)
6. **Updates**: Next launch or manual reload

**Features**:
- 8-color neon gradient (bottom to top)
- High-energy animations (fast drops, smooth transitions)
- Optimized for EDM/electronic music
- Customizable bars, gravity, sensitivity, framerate

**Customization** (in config.toml):
```toml
[[programs]]
name = "omarcava"
enabled = true
[programs.variables]
bars = 0               # 0 = auto-adjust to terminal width (recommended)
gravity = 85           # Higher = faster drops (neon flicker)
integral = 55          # Higher = smoother transitions
monstercat = 35        # Bass emphasis (club feel)
framerate = 60         # 30-60 FPS
```

### Omarclock (tclock Wrapper)

**Clean Cyberpunk Terminal Clock**

1. **Template**: Minimal shell script wrapper for tclock with theme colors
2. **Color Injection**: Applies theme's cyan color to tclock
3. **Deployment**: Creates executable script at `~/.local/bin/omarclock`
4. **Usage**: Run `omarclock` to launch themed clock
5. **Updates**: Instant on next launch

**Features**:
- Clean, minimal design (no borders, no extra text)
- Dynamic color from current Omarchy theme (bright cyan)
- No circular background - pure cyberpunk aesthetics
- Supports command-line arguments for customization
- Automatic bar display adjustments

**Usage**:
```bash
# Launch clean clock (default: no seconds, no border)
omarclock

# Show seconds
omarclock --seconds

# Add box outline
omarclock --box

# Add colored disc background
omarclock --disc

# 24-hour format
omarclock --24

# Analog style
omarclock --analog

# Countdown timer
omarclock --countdown 5m

# Pass any tclock flag directly
omarclock -bounce 2
```

### Color Extraction

The generator extracts colors from your Omarchy theme files in priority order:

1. **alacritty.toml**: ANSI colors (black, red, green, yellow, blue, magenta, cyan, white + bright variants)
2. **btop.theme**: Btop color definitions
3. **custom_theme.json**: Custom color palette

**Important Note**: Your theme's turquoise color (#8FECD5) is stored in the **green** color slot, not cyan. This is by design in the Omarchy theme system.

## Themes

### Omarcord

Based on [System24](https://github.com/refact0r/system24) theme for Discord. Features:
- Clean, minimal TUI-inspired design
- Terminal ANSI color scheme
- Turquoise accents matching Omarchy
- ASCII loader with "Omarcord" branding
- Optimized for readability

### Omarchify

Based on [text theme](https://github.com/spicetify/spicetify-themes/tree/master/text) for Spotify. Features:
- Text-focused, minimal interface
- Turquoise accents on black/white base
- Synchronized with Discord theme
- ASCII banner with "omarchi-fy" branding
- Multiple color schemes included in base theme

### Omarcava

Cyberpunk 2077-inspired audio visualizer theme for Cava. Features:
- 8-color vertical neon gradient (purple → cyan → turquoise)
- High-energy animations with fast drops (neon flicker effect)
- Optimized for EDM/electronic music visualization
- Full frequency spectrum coverage (50Hz-10kHz)
- Auto-adjusting bar count (prevents window width errors)
- Customizable gravity, sensitivity, and framerate
- Auto-reloads on theme change

### Omarclock

Clean cyberpunk wrapper for tclock terminal clock. Features:
- Minimal design with no borders or background circles
- Dynamic cyan color from current Omarchy theme
- Clean, distraction-free time display
- Support for all tclock features via pass-through arguments
- Instant theme updates on launch
- Pure cyberpunk aesthetics - just clean digits

### Omarvscode

VS Code theme with vibrant aesthetic. Features:
- Full UI theming (activity bar, sidebar, tabs, editor, terminal)
- Signature bright yellow status bar (bright red when debugging)
- Terminal colors match Omarchy ANSI palette
- Syntax highlighting with theme colors
- Turquoise accents throughout the interface
- Instant updates on theme change (requires window reload)

**Installation & Activation**:
1. Run: `omarchy-theme-gen once` to generate theme (enabled by default)
2. Reload VS Code: `Ctrl+Shift+P` → "Developer: Reload Window"
3. Select theme: `Ctrl+Shift+P` → "Preferences: Color Theme" → "Omarvscode"

**How it works**:
- Creates VS Code extension at `~/.vscode/extensions/local.theme-omarvscode/`
- Extension structure:
  - `package.json`: Extension metadata
  - `themes/omarvscode-color-theme.json`: Theme definition with Omarchy colors
- Updates instantly on theme change (reload window to apply)
- Works globally across all VS Code workspaces

## Troubleshooting

### Theme Not Updating

```bash
# Check if service is running
systemctl --user status omarchy-theme-gen

# Manually regenerate themes
omarchy-theme-gen once

# Check logs for errors
journalctl --user -u omarchy-theme-gen -n 50
```

### Colors Look Wrong

1. Check that your Omarchy theme has valid color definitions
2. Verify color_priority in config.toml
3. Remember: turquoise is in the **green** slot, not cyan
4. Check which file was used: logs show "Extracted colors from X"

### Vencord Theme Not Appearing

1. Ensure Vencord is installed: Check for `~/.config/Vencord/`
2. Launch Discord/Vesktop to initialize Vencord
3. Check that `omarcord.theme.css` exists in Vencord themes folder
4. Manually enable in Vencord settings if needed
5. Check logs for file write errors

### Spicetify Theme Not Applying

```bash
# Check Spicetify installation
spicetify --version

# Manually apply theme (with restart)
spicetify config current_theme text
spicetify config color_scheme Omarchify
spicetify apply

# Restore if needed
spicetify restore
spicetify apply
```

### Spicetify CLI Not Detected

The generator checks these locations for spicetify:
- In PATH (via `command -v spicetify`)
- `~/.spicetify/spicetify`
- `~/.local/bin/spicetify`
- `/usr/local/bin/spicetify`

If installed elsewhere, ensure it's in one of these locations or add to PATH.

### Cava Not Detected

```bash
# Verify Cava is installed
which cava
cava -v

# If not installed:
sudo pacman -S cava  # Arch
sudo apt install cava  # Debian/Ubuntu
brew install cava  # macOS
```

### Cava Colors Not Appearing

1. **Check terminal supports 24-bit color (truecolor)**:
   ```bash
   # Test truecolor support
   awk 'BEGIN{
       s="/\\/\\/\\/\\/\\"; s=s s s s s s s s;
       for (colnum = 0; colnum<77; colnum++) {
           r = 255-(colnum*255/76);
           g = (colnum*510/76);
           b = (colnum*255/76);
           if (g>255) g = 510-g;
           printf "\033[48;2;%d;%d;%dm", r,g,b;
           printf "\033[38;2;%d;%d;%dm", 255-r,255-g,255-b;
           printf "%s\033[0m", substr(s,colnum+1,1);
       }
       printf "\n";
   }'
   ```
2. **Check config was deployed**: `cat ~/.config/cava/config`
3. **Verify colors have correct format**: Should be hex like `'#bd00ff'`
4. **Regenerate if needed**: `omarchy-theme-gen once` (omarcava is enabled by default)

### Cava Theme Not Updating

Cava doesn't auto-reload config. To see new theme:
```bash
# Press 'r' in running Cava to reload
# OR restart Cava
pkill cava && cava
```

### Cava Visualization Issues

**Too sensitive / Not sensitive enough**:
```toml
# In config.toml
[programs.variables]
autosens = 0       # Disable auto-sensitivity
sensitivity = 150  # Manual sensitivity (decrease for less sensitive)
```

**Bars drop too slowly (not snappy enough)**:
```toml
[programs.variables]
gravity = 100  # Increase for faster drops (cyberpunk flicker)
```

**Visualization looks muddy/blurry**:
```toml
[programs.variables]
integral = 40      # Decrease for less smoothing
monstercat = 25    # Decrease bass emphasis
```

**Want more/fewer bars**:
```toml
[programs.variables]
bars = 0   # 0 = auto-adjust (recommended), or set specific number like 32-64
```

**Note**: Setting `bars = 0` (default) automatically adjusts the number of bars to fit your terminal width, preventing "window too narrow" errors.

### tclock Not Detected

```bash
# Verify tclock is installed
which tclock
tclock --help

# If not installed, install via cargo:
cargo install tclock

# Or build from source:
git clone https://github.com/nwrenger/tclock
cd tclock
cargo install --path .

# Ensure ~/.cargo/bin is in PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Omarclock Not Working

1. **Check wrapper exists**: `ls -la ~/.local/bin/omarclock`
2. **Check executable**: `file ~/.local/bin/omarclock` (should say "shell script")
3. **Check PATH includes ~/.local/bin**: `echo $PATH`
4. **Test directly**: `~/.local/bin/omarclock`
5. **Regenerate if needed**: `omarchy-theme-gen once` (omarclock is enabled by default)

### Omarclock Colors Wrong

The wrapper script uses your theme's bright cyan color. If colors look wrong:
1. Check `omarclock` script has correct color value: `cat ~/.local/bin/omarclock | grep PRIMARY_COLOR`
2. Regenerate: `omarchy-theme-gen once`
3. Verify theme has `bright_cyan` color defined in Omarchy theme
4. Try running directly: `tclock -color "#00d4ff"` to test tclock itself

### VS Code Theme Not Appearing

1. **Check extension exists**: `ls -la ~/.vscode/extensions/local.theme-omarvscode/`
2. **Verify extension is recognized**: `code --list-extensions | grep omarvscode`
3. **Check files exist**:
   ```bash
   ls ~/.vscode/extensions/local.theme-omarvscode/package.json
   ls ~/.vscode/extensions/local.theme-omarvscode/themes/omarvscode-color-theme.json
   ```
4. **Regenerate**: `omarchy-theme-gen once`
5. **Reload VS Code**: `Ctrl+Shift+P` → "Developer: Reload Window"
6. **Select theme**: `Ctrl+Shift+P` → "Preferences: Color Theme" → "Omarvscode"

### VS Code Theme Not Updating

VS Code doesn't auto-reload themes. After theme changes:
```bash
# Regenerate theme
omarchy-theme-gen once

# Reload VS Code window
# Ctrl+Shift+P → "Developer: Reload Window"
```

### VS Code Theme Colors Wrong

1. **Check theme file**: `cat ~/.vscode/extensions/local.theme-omarvscode/themes/omarvscode-color-theme.json`
2. **Verify colors format**: Should be hex like `"#ffe64d"`
3. **Check Omarchy theme** has all color definitions
4. **Regenerate**: `omarchy-theme-gen once` and reload VS Code

## Development

### Building from Source

```bash
cd Generator
cargo build --release
```

### Running Tests

```bash
cd Generator
cargo test
```

### Template Development

Templates use [Tera](https://tera.netlify.app/) syntax (similar to Jinja2):

**Available Variables**:
- Color values: `{{ background }}`, `{{ foreground }}`, etc.
- Hex without #: `{{ background_hex }}` for INI files
- All ANSI colors: black, red, green, yellow, blue, magenta, cyan, white
- Bright variants: `{{ bright_green }}`, etc.

Example:
```css
--bg-color: {{ background | default(value="#1e1e2e") }};
```

### Adding New Programs

1. Create template in `Generator/templates/`
2. Add program config to `config.toml`:
```toml
[[programs]]
name = "yourprogram"
enabled = true
output_file = "theme.css"
template = "yourprogram"
```
3. Optionally add detection in `detector.rs`
4. Optionally add activation in `activator.rs`

## Architecture

### Core Components

- **main.rs**: CLI argument parsing and command dispatch
- **config.rs**: Configuration file handling (TOML)
- **detector.rs**: Detects installed programs (Vencord, Spicetify)
- **extractor.rs**: Extracts colors from Omarchy theme files
- **generator.rs**: Orchestrates theme generation and deployment
- **templates.rs**: Tera template rendering engine
- **activator.rs**: Activates themes in target programs
- **watcher.rs**: File system watching for theme changes
- **linker.rs**: Symlink management utilities

### Data Flow

```
Omarchy Theme Change
    ↓
Watcher Detects Change
    ↓
Extract Colors from Theme Files
    ↓
Render Templates with Colors
    ↓
Deploy Generated Files
    ↓
Create/Update Symlinks
    ↓
Activate Themes in Programs
    ↓
Done
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Test thoroughly
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

[MIT License](LICENSE) - see LICENSE file for details.

This project incorporates code from:
- **System24**: MIT License - Copyright (c) 2025 refact0r
- **Spicetify text theme**: MIT License - Copyright (c) 2019 morpheusthewhite

See respective LICENSE files in subdirectories for details.

## Credits

- **System24**: Base Discord theme by [refact0r](https://github.com/refact0r/system24)
- **text theme**: Base Spotify theme by [Spicetify community](https://github.com/spicetify/spicetify-themes)
- **Omarchy**: The foundation theme system by szamski
- **Vencord**: Discord client modification - [vencord.dev](https://vencord.dev/)
- **Spicetify**: Spotify theming CLI - [spicetify.app](https://spicetify.app/)

## Links

- **Omarchy**: [github.com/basecamp/omarchy](https://github.com/basecamp/omarchy)
- **System24**: [github.com/refact0r/system24](https://github.com/refact0r/system24)
- **Vencord**: [vencord.dev](https://vencord.dev/)
- **Spicetify**: [spicetify.app](https://spicetify.app/)
- **Report Issues**: [github.com/szamski/omarchy-theme-gen/issues](https://github.com/szamski/omarchy-theme-gen/issues)

## Acknowledgments

Built with:
- Rust - Systems programming language
- Tera - Template engine
- Notify - File system watcher
- Serde - Serialization framework
- Tokio - Async runtime
- Tracing - Logging framework
