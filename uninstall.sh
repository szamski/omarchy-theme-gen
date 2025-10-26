#!/bin/bash
# Omarchy Theme Generator - Uninstallation Script
# Removes: Generator + Configuration + Service + Generated Files

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
GENERATED_DIR="$ACTUAL_HOME/.config/omarchy-themes/generated"
BACKUP_DIR="$ACTUAL_HOME/.config/omarchy-themes/backups"
SYSTEMD_USER_DIR="$ACTUAL_HOME/.config/systemd/user"
SERVICE_FILE="$SYSTEMD_USER_DIR/$BINARY_NAME.service"

# Function to print status messages
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warning() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }
section() { echo -e "\n${CYAN}=== $1 ===${NC}\n"; }

section "Omarchy Theme Generator - Uninstallation"

echo "This will remove:"
echo "  • Theme generator binary"
echo "  • Configuration files"
echo "  • Systemd service"
echo "  • Generated theme files"
echo "  • Symlinks (Spicetify themes)"
echo ""
echo "This will NOT remove:"
echo "  • Spicetify installation"
echo "  • Vencord installation"
echo "  • Your Omarchy themes"
echo ""

read -p "Continue with uninstallation? [y/N]: " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    info "Uninstallation cancelled"
    exit 0
fi

#############################################
# PART 1: Stop and Remove Service
#############################################

section "Part 1: Removing Systemd Service"

if [ -f "$SERVICE_FILE" ]; then
    info "Stopping service..."
    systemctl --user stop "$BINARY_NAME.service" 2>/dev/null || true

    info "Disabling service..."
    systemctl --user disable "$BINARY_NAME.service" 2>/dev/null || true

    info "Removing service file..."
    rm -f "$SERVICE_FILE"

    systemctl --user daemon-reload
    success "Service removed"
else
    info "Service not found, skipping"
fi

#############################################
# PART 2: Remove Symlinks
#############################################

section "Part 2: Removing Symlinks"

# Remove Spicetify symlink if it exists
SPICETIFY_SYMLINK="$ACTUAL_HOME/.config/spicetify/Themes/text/color.ini"
if [ -L "$SPICETIFY_SYMLINK" ]; then
    info "Removing Spicetify symlink..."
    rm -f "$SPICETIFY_SYMLINK"
    success "Spicetify symlink removed"
else
    info "No Spicetify symlink found"
fi

#############################################
# PART 3: Remove Binary
#############################################

section "Part 3: Removing Binary"

BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
if [ -f "$BINARY_PATH" ]; then
    info "Removing binary from $BINARY_PATH..."
    rm -f "$BINARY_PATH"
    success "Binary removed"
else
    info "Binary not found, skipping"
fi

#############################################
# PART 4: Remove Configuration
#############################################

section "Part 4: Removing Configuration"

if [ -d "$CONFIG_DIR" ]; then
    read -p "Remove configuration directory? [Y/n]: " remove_config

    if [[ ! "$remove_config" =~ ^[Nn]$ ]]; then
        # Create backup of config
        if [ -f "$CONFIG_DIR/config.toml" ]; then
            BACKUP_CONFIG_DIR="$ACTUAL_HOME/.config/omarchy-themes/config-backup"
            mkdir -p "$BACKUP_CONFIG_DIR"
            cp "$CONFIG_DIR/config.toml" "$BACKUP_CONFIG_DIR/config.toml.backup"
            info "Config backed up to: $BACKUP_CONFIG_DIR/config.toml.backup"
        fi

        rm -rf "$CONFIG_DIR"
        success "Configuration removed"
    else
        info "Configuration kept"
    fi
else
    info "Configuration directory not found"
fi

#############################################
# PART 5: Remove Generated Files
#############################################

section "Part 5: Removing Generated Files"

if [ -d "$GENERATED_DIR" ]; then
    read -p "Remove generated theme files? [Y/n]: " remove_generated

    if [[ ! "$remove_generated" =~ ^[Nn]$ ]]; then
        rm -rf "$GENERATED_DIR"
        success "Generated files removed"
    else
        info "Generated files kept"
    fi
else
    info "Generated files directory not found"
fi

if [ -d "$BACKUP_DIR" ]; then
    read -p "Remove backup files? [Y/n]: " remove_backups

    if [[ ! "$remove_backups" =~ ^[Nn]$ ]]; then
        rm -rf "$BACKUP_DIR"
        success "Backup files removed"
    else
        info "Backup files kept"
    fi
fi

#############################################
# PART 6: Clean Vencord Theme (optional)
#############################################

section "Part 6: Clean Vencord Theme (Optional)"

VENCORD_THEME="$ACTUAL_HOME/.config/Vencord/themes/omarcord.theme.css"
if [ -f "$VENCORD_THEME" ]; then
    read -p "Remove Omarcord theme from Vencord? [y/N]: " remove_vencord_theme

    if [[ "$remove_vencord_theme" =~ ^[Yy]$ ]]; then
        rm -f "$VENCORD_THEME"
        success "Vencord theme removed"

        # Disable in Vencord settings
        VENCORD_SETTINGS="$ACTUAL_HOME/.config/Vencord/settings/settings.json"
        if [ -f "$VENCORD_SETTINGS" ]; then
            info "Note: You may need to manually disable the theme in Vencord settings"
        fi
    fi
else
    info "Vencord theme not found"
fi

#############################################
# PART 7: Clean Spicetify Theme (optional)
#############################################

section "Part 7: Clean Spicetify Theme (Optional)"

SPICETIFY_TEXT_THEME="$ACTUAL_HOME/.config/spicetify/Themes/text"
if [ -d "$SPICETIFY_TEXT_THEME" ]; then
    read -p "Remove text theme from Spicetify? [y/N]: " remove_spicetify_theme

    if [[ "$remove_spicetify_theme" =~ ^[Yy]$ ]]; then
        rm -rf "$SPICETIFY_TEXT_THEME"
        success "Spicetify text theme removed"

        if command -v spicetify &> /dev/null; then
            info "Resetting Spicetify to default..."
            spicetify config current_theme "" 2>/dev/null || true
            spicetify config color_scheme "" 2>/dev/null || true
            spicetify apply 2>/dev/null || true
            success "Spicetify reset to default"
        fi
    fi
else
    info "Spicetify text theme not found"
fi

#############################################
# Summary
#############################################

section "Uninstallation Complete!"

echo "Removed:"
[ ! -f "$BINARY_PATH" ] && echo "  ${GREEN}✓${NC} Binary"
[ ! -f "$SERVICE_FILE" ] && echo "  ${GREEN}✓${NC} Systemd service"
[ ! -d "$CONFIG_DIR" ] && echo "  ${GREEN}✓${NC} Configuration"
[ ! -L "$SPICETIFY_SYMLINK" ] && echo "  ${GREEN}✓${NC} Symlinks"

echo ""
echo "Still installed (not removed by this script):"
command -v spicetify &> /dev/null && echo "  ${BLUE}•${NC} Spicetify"
[ -d "$ACTUAL_HOME/.config/Vencord" ] && echo "  ${BLUE}•${NC} Vencord"

echo ""
echo "To reinstall, run: ${YELLOW}./install.sh${NC}"
echo ""
