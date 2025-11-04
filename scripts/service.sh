#!/bin/bash

# Web Watcher Alert - Service Control Script
# Control the background monitoring service

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PLIST_DEST="$HOME/Library/LaunchAgents/com.webwatcheralert.plist"
LOG_DIR="$HOME/.local/share/web-watcher-alert/logs"
SERVICE_NAME="com.webwatcheralert"

# Check if service is installed
check_installed() {
    if [ ! -f "$PLIST_DEST" ]; then
        echo -e "${RED}✗ Service not installed${NC}"
        echo -e "${YELLOW}  Run: ./scripts/install-service.sh${NC}"
        exit 1
    fi
}

# Check if service is running
is_running() {
    launchctl list | grep -q "$SERVICE_NAME"
}

# Show usage
show_usage() {
    echo -e "${BLUE}Web Watcher Alert - Service Control${NC}"
    echo ""
    echo -e "${GREEN}Usage:${NC}"
    echo -e "  ./scripts/service.sh ${YELLOW}<command>${NC}"
    echo ""
    echo -e "${GREEN}Commands:${NC}"
    echo -e "  ${YELLOW}start${NC}       Start the background monitoring service"
    echo -e "  ${YELLOW}stop${NC}        Stop the background monitoring service"
    echo -e "  ${YELLOW}restart${NC}     Restart the service"
    echo -e "  ${YELLOW}status${NC}      Check if the service is running"
    echo -e "  ${YELLOW}logs${NC}        View recent logs (last 50 lines)"
    echo -e "  ${YELLOW}logs-tail${NC}   Follow logs in real-time"
    echo -e "  ${YELLOW}logs-stdout${NC} View full stdout log"
    echo -e "  ${YELLOW}logs-stderr${NC} View full stderr log"
    echo -e "  ${YELLOW}uninstall${NC}   Remove the service"
    echo ""
}

# Start service
start_service() {
    check_installed

    if is_running; then
        echo -e "${YELLOW}⚠ Service is already running${NC}"
        exit 0
    fi

    echo -e "${BLUE}Starting service...${NC}"
    launchctl start "$SERVICE_NAME"
    sleep 1

    if is_running; then
        echo -e "${GREEN}✓ Service started successfully${NC}"
        echo -e "${YELLOW}  View logs: ./scripts/service.sh logs${NC}"
    else
        echo -e "${RED}✗ Failed to start service${NC}"
        echo -e "${YELLOW}  Check logs: ./scripts/service.sh logs-stderr${NC}"
        exit 1
    fi
}

# Stop service
stop_service() {
    check_installed

    if ! is_running; then
        echo -e "${YELLOW}⚠ Service is not running${NC}"
        exit 0
    fi

    echo -e "${BLUE}Stopping service...${NC}"
    launchctl stop "$SERVICE_NAME"
    sleep 1

    if ! is_running; then
        echo -e "${GREEN}✓ Service stopped successfully${NC}"
    else
        echo -e "${RED}✗ Failed to stop service${NC}"
        exit 1
    fi
}

# Restart service
restart_service() {
    echo -e "${BLUE}Restarting service...${NC}"
    stop_service 2>/dev/null || true
    sleep 1
    start_service
}

# Check status
check_status() {
    check_installed

    echo -e "${BLUE}Service Status:${NC}"
    echo ""

    if is_running; then
        echo -e "  Status: ${GREEN}● Running${NC}"

        # Get PID if possible
        PID=$(launchctl list | grep "$SERVICE_NAME" | awk '{print $1}')
        if [ "$PID" != "-" ]; then
            echo -e "  PID: ${YELLOW}$PID${NC}"
        fi
    else
        echo -e "  Status: ${RED}○ Stopped${NC}"
    fi

    echo ""
    echo -e "  Plist: $PLIST_DEST"
    echo -e "  Logs: $LOG_DIR"
    echo ""
}

# View logs
view_logs() {
    check_installed

    if [ ! -d "$LOG_DIR" ]; then
        echo -e "${RED}✗ Log directory not found: $LOG_DIR${NC}"
        exit 1
    fi

    echo -e "${BLUE}Recent logs (last 50 lines):${NC}"
    echo -e "${YELLOW}────────────────────────────────────────────${NC}"

    if [ -f "$LOG_DIR/stdout.log" ]; then
        tail -n 50 "$LOG_DIR/stdout.log"
    else
        echo -e "${YELLOW}No stdout log found yet${NC}"
    fi
}

# Tail logs in real-time
tail_logs() {
    check_installed

    if [ ! -f "$LOG_DIR/stdout.log" ]; then
        echo -e "${YELLOW}⚠ Log file not found yet. Service may not have started.${NC}"
        echo -e "${YELLOW}  Creating log file...${NC}"
        mkdir -p "$LOG_DIR"
        touch "$LOG_DIR/stdout.log"
    fi

    echo -e "${BLUE}Following logs (Ctrl+C to stop):${NC}"
    echo -e "${YELLOW}────────────────────────────────────────────${NC}"
    tail -f "$LOG_DIR/stdout.log"
}

# View full stdout log
view_stdout() {
    check_installed

    if [ -f "$LOG_DIR/stdout.log" ]; then
        less "$LOG_DIR/stdout.log"
    else
        echo -e "${YELLOW}No stdout log found${NC}"
    fi
}

# View full stderr log
view_stderr() {
    check_installed

    if [ -f "$LOG_DIR/stderr.log" ]; then
        less "$LOG_DIR/stderr.log"
    else
        echo -e "${YELLOW}No stderr log found${NC}"
    fi
}

# Uninstall service
uninstall_service() {
    check_installed

    echo -e "${YELLOW}⚠ This will remove the service (logs will be preserved)${NC}"
    read -p "Continue? (y/N) " -n 1 -r
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Cancelled${NC}"
        exit 0
    fi

    echo -e "${BLUE}Uninstalling service...${NC}"

    # Stop if running
    if is_running; then
        launchctl stop "$SERVICE_NAME"
    fi

    # Unload
    launchctl unload "$PLIST_DEST" 2>/dev/null || true

    # Remove plist
    rm -f "$PLIST_DEST"

    echo -e "${GREEN}✓ Service uninstalled${NC}"
    echo -e "${YELLOW}  Logs are still available at: $LOG_DIR${NC}"
    echo -e "${YELLOW}  To reinstall, run: ./scripts/install-service.sh${NC}"
}

# Main command handler
case "${1:-}" in
    start)
        start_service
        ;;
    stop)
        stop_service
        ;;
    restart)
        restart_service
        ;;
    status)
        check_status
        ;;
    logs)
        view_logs
        ;;
    logs-tail)
        tail_logs
        ;;
    logs-stdout)
        view_stdout
        ;;
    logs-stderr)
        view_stderr
        ;;
    uninstall)
        uninstall_service
        ;;
    *)
        show_usage
        exit 1
        ;;
esac
