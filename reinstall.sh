#!/bin/bash
# Omarchy Theme Generator - Reinstall Script
# Uninstalls and reinstalls the generator (useful for updates)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print status messages
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warning() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }
section() { echo -e "\n${CYAN}=== $1 ===${NC}\n"; }

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if scripts exist
if [ ! -f "$SCRIPT_DIR/uninstall.sh" ]; then
    error "uninstall.sh not found in $SCRIPT_DIR"
    exit 1
fi

if [ ! -f "$SCRIPT_DIR/install.sh" ]; then
    error "install.sh not found in $SCRIPT_DIR"
    exit 1
fi

section "Omarchy Theme Generator - Reinstall"

echo "This will:"
echo "  1. Uninstall the current installation"
echo "  2. Rebuild from source"
echo "  3. Install fresh version"
echo ""
echo "Your Spicetify, Vencord, Cava, tclock, and VS Code theme will remain intact."
echo ""

read -p "Continue with reinstall? [Y/n]: " confirm
if [[ "$confirm" =~ ^[Nn]$ ]]; then
    info "Reinstall cancelled"
    exit 0
fi

#############################################
# PART 1: Uninstall
#############################################

section "Part 1: Uninstalling Current Version"

# Run uninstall with automatic confirmations
# y = remove config (with backup)
# y = remove generated files
# y = remove backups
# n = don't remove Vencord theme
# n = don't remove Spicetify theme
# n = don't remove Cava config
# n = don't remove tclock wrapper
# n = don't remove VS Code theme
echo -e "y\ny\ny\nn\nn\nn\nn\nn" | bash "$SCRIPT_DIR/uninstall.sh"

if [ $? -ne 0 ]; then
    error "Uninstall failed!"
    exit 1
fi

success "Uninstall complete"

#############################################
# PART 2: Clean Build
#############################################

section "Part 2: Cleaning Build Artifacts"

if [ -d "$SCRIPT_DIR/Generator/target" ]; then
    info "Cleaning previous build..."
    cd "$SCRIPT_DIR/Generator"

    # Try cargo clean first
    if cargo clean 2>/dev/null; then
        success "Build artifacts cleaned"
    else
        warning "Cargo clean failed (permission issue), trying alternative method..."

        # Try with sudo if not root
        if [ "$EUID" -ne 0 ]; then
            info "Attempting to fix permissions and clean..."
            if sudo chmod -R u+w target 2>/dev/null && cargo clean 2>/dev/null; then
                success "Build artifacts cleaned with fixed permissions"
            else
                warning "Could not clean build artifacts, removing target directory..."
                # Last resort: remove the entire target directory
                if sudo rm -rf target 2>/dev/null; then
                    success "Target directory removed"
                else
                    warning "Could not remove target directory, continuing anyway..."
                fi
            fi
        else
            # If running as root, just remove it
            rm -rf target 2>/dev/null || true
            success "Build artifacts removed"
        fi
    fi
else
    info "No build artifacts to clean"
fi

#############################################
# PART 3: Install Fresh
#############################################

section "Part 3: Installing Fresh Version"

# Run install script
bash "$SCRIPT_DIR/install.sh"

if [ $? -ne 0 ]; then
    error "Installation failed!"
    exit 1
fi

#############################################
# Summary
#############################################

section "Reinstall Complete!"

echo "✓ Old version uninstalled"
echo "✓ Build artifacts cleaned"
echo "✓ Fresh version installed"
echo ""
echo "The generator is now ready to use!"
echo ""
echo "Quick commands:"
echo "  ${YELLOW}omarchy-theme-gen once${NC}        # Generate themes now"
echo "  ${YELLOW}omarchy-theme-gen watch${NC}       # Watch for changes"
echo "  ${YELLOW}systemctl --user status omarchy-theme-gen${NC}  # Check service"
echo ""
