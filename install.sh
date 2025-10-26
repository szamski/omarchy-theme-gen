#!/bin/bash
# Omarchy Theme Generator - Comprehensive Installation Script
# Installs: Generator + Spicetify + Vencord + Themes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Detect user and home
ACTUAL_USER="${SUDO_USER:-$USER}"
IS_ROOT=false
if [ "$EUID" -eq 0 ]; then
    IS_ROOT=true
fi

if [ "$IS_ROOT" = true ] && [ -n "$SUDO_USER" ]; then
    ACTUAL_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
else
    ACTUAL_HOME="$HOME"
fi

# Configuration
BINARY_NAME="omarchy-theme-gen"
INSTALL_DIR="$ACTUAL_HOME/.local/bin"
CONFIG_DIR="$ACTUAL_HOME/.config/omarchy-theme-watcher"
SYSTEMD_USER_DIR="$ACTUAL_HOME/.config/systemd/user"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Status tracking
INSTALLED_GENERATOR=false
INSTALLED_SPICETIFY=false
INSTALLED_VENCORD=false

# Function to print status messages
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warning() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }
section() { echo -e "\n${CYAN}=== $1 ===${NC}\n"; }

# Check command exists
command_exists() {
    command -v "$1" &> /dev/null
}

section "Omarchy Theme Generator - Full Installation"

#############################################
# PART 1: Install Generator
#############################################

section "Part 1: Installing Theme Generator"

# Check if running from project directory
if [ ! -f "$PROJECT_DIR/Generator/Cargo.toml" ]; then
    error "Generator directory not found. Please run from project root."
    exit 1
fi

# Check if cargo is installed
if ! command_exists cargo; then
    error "Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"

# Build the project
info "Building release binary..."
cd "$PROJECT_DIR/Generator"
cargo build --release

if [ ! -f "target/release/$BINARY_NAME" ]; then
    error "Build failed - binary not found"
    exit 1
fi

# Install binary
info "Installing binary to $INSTALL_DIR..."
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
success "Generator installed to $INSTALL_DIR/$BINARY_NAME"
INSTALLED_GENERATOR=true

# Initialize configuration
info "Initializing configuration..."
"$INSTALL_DIR/$BINARY_NAME" init-config
success "Configuration created at $CONFIG_DIR/config.toml"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    warning "$INSTALL_DIR is not in your PATH"
    echo -e "  Add to ~/.bashrc or ~/.zshrc: ${YELLOW}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
fi

#############################################
# PART 2: Install Spicetify (if needed)
#############################################

section "Part 2: Spicetify Setup"

if command_exists spicetify; then
    success "Spicetify is already installed"
    spicetify --version
else
    echo "Spicetify is not installed."
    read -p "Do you want to install Spicetify? [Y/n]: " install_spicetify

    if [[ ! "$install_spicetify" =~ ^[Nn]$ ]]; then
        info "Installing Spicetify..."
        curl -fsSL https://raw.githubusercontent.com/spicetify/cli/main/install.sh | sh

        if command_exists spicetify; then
            success "Spicetify installed successfully"
            INSTALLED_SPICETIFY=true

            # Apply spicetify
            info "Applying Spicetify to Spotify..."
            spicetify backup apply
        else
            warning "Spicetify installation may have failed. Please check manually."
        fi
    fi
fi

# Install text theme for Spicetify
if command_exists spicetify; then
    SPICETIFY_THEMES_DIR="$ACTUAL_HOME/.config/spicetify/Themes"
    TEXT_THEME_SOURCE="$PROJECT_DIR/Omarchify/text"
    TEXT_THEME_DEST="$SPICETIFY_THEMES_DIR/text"

    if [ -d "$TEXT_THEME_SOURCE" ]; then
        info "Installing text theme for Spicetify..."
        mkdir -p "$SPICETIFY_THEMES_DIR"

        # Copy text theme (base files: user.css, README, screenshots)
        cp -r "$TEXT_THEME_SOURCE" "$SPICETIFY_THEMES_DIR/"

        # Remove the original color.ini (will be generated/symlinked by our generator)
        rm -f "$TEXT_THEME_DEST/color.ini"

        success "Text theme installed to $TEXT_THEME_DEST"
    else
        warning "Text theme source not found at $TEXT_THEME_SOURCE"
    fi
fi

#############################################
# PART 3: Install Vencord (if needed)
#############################################

section "Part 3: Vencord Setup"

VENCORD_DIR="$ACTUAL_HOME/.config/Vencord"

if [ -d "$VENCORD_DIR" ]; then
    success "Vencord is already installed at $VENCORD_DIR"
else
    echo "Vencord is not installed."
    echo "Vencord requires Discord/Vesktop to be installed first."

    read -p "Do you want to install Vencord? [Y/n]: " install_vencord

    if [[ ! "$install_vencord" =~ ^[Nn]$ ]]; then
        info "Installing Vencord..."
        echo ""
        echo "Vencord installation options:"
        echo "  1) Install via web installer (recommended)"
        echo "  2) Manual installation instructions"
        read -p "Choose option [1-2]: " vencord_choice

        case $vencord_choice in
            1)
                info "Opening Vencord web installer..."
                if command_exists xdg-open; then
                    xdg-open "https://vencord.dev/download/" 2>/dev/null
                fi
                echo ""
                echo "Please:"
                echo "  1. Download and install Vencord from the website"
                echo "  2. Launch Discord/Vesktop with Vencord"
                echo "  3. Press Enter when done"
                read -p "Press Enter to continue..."

                if [ -d "$VENCORD_DIR" ]; then
                    success "Vencord detected at $VENCORD_DIR"
                    INSTALLED_VENCORD=true
                else
                    warning "Vencord directory not found. You may need to launch Discord first."
                fi
                ;;
            2)
                info "Manual Vencord installation:"
                echo ""
                echo "  1. Visit: https://vencord.dev/download/"
                echo "  2. Download Vencord installer"
                echo "  3. Run the installer for your Discord client"
                echo "  4. Launch Discord/Vesktop"
                echo "  5. Re-run this script to continue"
                echo ""
                ;;
        esac
    fi
fi

#############################################
# PART 4: Systemd Service Setup
#############################################

section "Part 4: Systemd Service Setup"

read -p "Install systemd user service for auto-start? [Y/n]: " install_service

if [[ ! "$install_service" =~ ^[Nn]$ ]]; then
    info "Creating systemd service..."
    mkdir -p "$SYSTEMD_USER_DIR"

    cat > "$SYSTEMD_USER_DIR/$BINARY_NAME.service" << EOF
[Unit]
Description=Omarchy Theme Generator
Documentation=https://github.com/yourusername/omarchy-theme-gen
After=graphical-session.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/$BINARY_NAME watch
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
EOF

    success "Service file created"

    systemctl --user daemon-reload

    read -p "Enable and start the service now? [Y/n]: " enable_service

    if [[ ! "$enable_service" =~ ^[Nn]$ ]]; then
        systemctl --user enable "$BINARY_NAME.service"
        systemctl --user start "$BINARY_NAME.service"
        success "Service enabled and started"

        echo ""
        systemctl --user status "$BINARY_NAME.service" --no-pager || true
    fi
fi

#############################################
# PART 5: First Run
#############################################

section "Part 5: Initial Theme Generation"

read -p "Generate themes for current Omarchy theme now? [Y/n]: " run_now

if [[ ! "$run_now" =~ ^[Nn]$ ]]; then
    info "Running theme generator..."
    "$INSTALL_DIR/$BINARY_NAME" once
fi

#############################################
# Installation Summary
#############################################

section "Installation Complete!"

echo "Installed Components:"
[ "$INSTALLED_GENERATOR" = true ] && echo "  ${GREEN}✓${NC} Theme Generator"
[ "$INSTALLED_SPICETIFY" = true ] && echo "  ${GREEN}✓${NC} Spicetify"
[ "$INSTALLED_VENCORD" = true ] && echo "  ${GREEN}✓${NC} Vencord"

echo ""
echo "File Locations:"
echo "  Binary:     ${YELLOW}$INSTALL_DIR/$BINARY_NAME${NC}"
echo "  Config:     ${YELLOW}$CONFIG_DIR/config.toml${NC}"
if command_exists spicetify; then
    echo "  Spicetify:  ${YELLOW}$ACTUAL_HOME/.config/spicetify/${NC}"
fi
if [ -d "$VENCORD_DIR" ]; then
    echo "  Vencord:    ${YELLOW}$VENCORD_DIR${NC}"
fi

echo ""
echo "${CYAN}Usage:${NC}"
echo "  ${YELLOW}$BINARY_NAME watch${NC}   # Watch for theme changes (or use systemd service)"
echo "  ${YELLOW}$BINARY_NAME once${NC}    # Generate for current theme"
echo "  ${YELLOW}$BINARY_NAME detect${NC}  # Detect installed programs"
echo "  ${YELLOW}$BINARY_NAME status${NC}  # Show system status"
echo "  ${YELLOW}$BINARY_NAME help${NC}    # Show all commands"

if [[ "$install_service" =~ ^[Yy]$ ]] || [[ ! "$install_service" =~ ^[Nn]$ ]]; then
    echo ""
    echo "${CYAN}Service Commands:${NC}"
    echo "  ${YELLOW}systemctl --user status $BINARY_NAME${NC}   # Check status"
    echo "  ${YELLOW}systemctl --user restart $BINARY_NAME${NC}  # Restart"
    echo "  ${YELLOW}journalctl --user -u $BINARY_NAME -f${NC}   # View logs"
fi

echo ""
success "Setup complete! Your themes will now auto-update with Omarchy changes."
echo ""
