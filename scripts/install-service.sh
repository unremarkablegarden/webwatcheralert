#!/bin/bash

# Web Watcher Alert - Service Installation Script
# This script installs the LaunchAgent for running the monitor in the background

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Web Watcher Alert - Service Installer   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Get absolute paths
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PLIST_TEMPLATE="$PROJECT_DIR/com.webwatcheralert.plist"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_DEST="$LAUNCH_AGENTS_DIR/com.webwatcheralert.plist"
LOG_DIR="$HOME/.local/share/web-watcher-alert/logs"
BINARY_PATH="$PROJECT_DIR/target/release/web-watcher-alert"

echo -e "${YELLOW}[1/6]${NC} Checking prerequisites..."

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}✗ Error: Binary not found at $BINARY_PATH${NC}"
    echo -e "${YELLOW}  Please build the project first:${NC}"
    echo -e "  cd $PROJECT_DIR"
    echo -e "  cargo build --release"
    exit 1
fi
echo -e "${GREEN}✓${NC} Binary found"

# Check if plist template exists
if [ ! -f "$PLIST_TEMPLATE" ]; then
    echo -e "${RED}✗ Error: Plist template not found at $PLIST_TEMPLATE${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} Plist template found"

echo ""
echo -e "${YELLOW}[2/6]${NC} Creating directories..."

# Create LaunchAgents directory if it doesn't exist
mkdir -p "$LAUNCH_AGENTS_DIR"
echo -e "${GREEN}✓${NC} LaunchAgents directory: $LAUNCH_AGENTS_DIR"

# Create log directory
mkdir -p "$LOG_DIR"
echo -e "${GREEN}✓${NC} Log directory: $LOG_DIR"

echo ""
echo -e "${YELLOW}[3/6]${NC} Stopping existing service (if running)..."
launchctl unload "$PLIST_DEST" 2>/dev/null || true
echo -e "${GREEN}✓${NC} Service stopped"

echo ""
echo -e "${YELLOW}[4/6]${NC} Generating plist file..."

# Replace placeholders in plist template
sed -e "s|BINARY_PATH_PLACEHOLDER|$BINARY_PATH|g" \
    -e "s|HOME_PATH_PLACEHOLDER|$HOME|g" \
    -e "s|LOG_PATH_PLACEHOLDER|$LOG_DIR|g" \
    "$PLIST_TEMPLATE" > "$PLIST_DEST"

echo -e "${GREEN}✓${NC} Plist installed: $PLIST_DEST"

echo ""
echo -e "${YELLOW}[5/6]${NC} Loading service..."
launchctl load "$PLIST_DEST"
echo -e "${GREEN}✓${NC} Service loaded (but not started yet)"

echo ""
echo -e "${YELLOW}[6/6]${NC} Making control script executable..."
chmod +x "$SCRIPT_DIR/service.sh"
echo -e "${GREEN}✓${NC} Control script ready"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Installation Complete! ✓          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo -e "  ${GREEN}Start monitoring:${NC}  ./scripts/service.sh start"
echo -e "  ${GREEN}Stop monitoring:${NC}   ./scripts/service.sh stop"
echo -e "  ${GREEN}Check status:${NC}      ./scripts/service.sh status"
echo -e "  ${GREEN}View logs:${NC}         ./scripts/service.sh logs"
echo -e "  ${GREEN}Uninstall:${NC}         ./scripts/service.sh uninstall"
echo ""
echo -e "${YELLOW}Note:${NC} The service will run in the background, even after closing the terminal."
echo -e "${YELLOW}      Make sure you have watchers configured in the TUI first!${NC}"
echo ""
