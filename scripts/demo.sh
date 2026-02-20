#!/usr/bin/env bash
# Dugout Demo Script
# Records a professional terminal demo showing Dugout's core features

set -e

# Colors and formatting
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'

# Configuration
DEMO_DIR="/tmp/dugout-demo"
# Try to find dugout binary: use env var, then PATH, then cargo bin, then local build
if [ -z "$DUGOUT_BIN" ]; then
    if command -v dugout &> /dev/null; then
        DUGOUT_BIN="dugout"
    elif [ -f "$HOME/.cargo/bin/dugout" ]; then
        DUGOUT_BIN="$HOME/.cargo/bin/dugout"
    elif [ -f "./target/release/dugout" ]; then
        DUGOUT_BIN="./target/release/dugout"
    elif [ -f "./target/debug/dugout" ]; then
        DUGOUT_BIN="./target/debug/dugout"
    else
        DUGOUT_BIN="dugout"  # fallback, will error later if not found
    fi
fi
TYPING_SPEED="${TYPING_SPEED:-0.05}"  # seconds per character
COMMAND_DELAY="${COMMAND_DELAY:-1.5}"  # delay before running command
OUTPUT_DELAY="${OUTPUT_DELAY:-1.0}"    # delay after output
RECORD_MODE="${RECORD_MODE:-false}"

# Helper: simulate typing (for visual effect)
type_command() {
    local cmd="$1"
    echo -ne "${BOLD}${CYAN}\$ ${RESET}"
    
    if [ "$RECORD_MODE" = "true" ]; then
        # Simulate typing character by character
        for ((i=0; i<${#cmd}; i++)); do
            echo -n "${cmd:$i:1}"
            sleep "$TYPING_SPEED"
        done
    else
        # Just print instantly for non-recording mode
        echo -n "$cmd"
    fi
    
    echo  # newline
    sleep "$COMMAND_DELAY"
}

# Helper: run command and show output
run_command() {
    local cmd="$1"
    type_command "$cmd"
    eval "$cmd"
    sleep "$OUTPUT_DELAY"
}

# Helper: print section header
section() {
    echo
    echo -e "${BOLD}${YELLOW}# $1${RESET}"
    sleep 0.5
}

# Helper: clear screen with delay
clear_screen() {
    sleep 1
    clear
}

# Cleanup function
cleanup() {
    if [ -d "$DEMO_DIR" ]; then
        rm -rf "$DEMO_DIR"
    fi
}

# Main demo sequence
main() {
    # Setup
    trap cleanup EXIT
    cleanup  # Clean any previous runs
    mkdir -p "$DEMO_DIR"
    cd "$DEMO_DIR"
    
    # Ensure we're using the right binary
    if ! command -v "$DUGOUT_BIN" &> /dev/null && [ ! -f "$DUGOUT_BIN" ]; then
        echo "Error: dugout binary not found at $DUGOUT_BIN"
        echo "Build it first with: cargo build --release"
        exit 1
    fi
    
    # Create a simple demo app
    mkdir -p demo-app
    cd demo-app
    
    # Show banner
    echo -e "${BOLD}${GREEN}"
    echo "╔══════════════════════════════════════════╗"
    echo "║                                          ║"
    echo "║         🔐  DUGOUT DEMO  🔐             ║"
    echo "║                                          ║"
    echo "║   Git-native secrets manager for devs    ║"
    echo "║                                          ║"
    echo "╚══════════════════════════════════════════╝"
    echo -e "${RESET}"
    sleep 2
    
    # Demo sequence
    section "Initialize a new vault"
    run_command "$DUGOUT_BIN init"
    
    section "Add some secrets"
    run_command "$DUGOUT_BIN set DATABASE_URL postgresql://localhost:5432/myapp"
    run_command "$DUGOUT_BIN set API_KEY sk_live_1234567890abcdef"
    run_command "$DUGOUT_BIN set STRIPE_SECRET sk_test_abcdef1234567890"
    
    section "Retrieve a secret"
    run_command "$DUGOUT_BIN get API_KEY"
    
    section "List all secret keys"
    run_command "$DUGOUT_BIN list"
    
    section "Create a simple app"
    cat > app.sh <<'EOF'
#!/bin/bash
echo "🚀 App starting with secrets..."
echo "DATABASE_URL: $DATABASE_URL"
echo "API_KEY: ${API_KEY:0:15}..."
echo "STRIPE_SECRET: ${STRIPE_SECRET:0:12}..."
echo "✅ All secrets loaded!"
EOF
    chmod +x app.sh
    
    # Show the app file
    type_command "cat app.sh"
    cat app.sh
    sleep "$OUTPUT_DELAY"
    
    section "Run app with secrets injected"
    run_command "$DUGOUT_BIN run -- ./app.sh"
    
    # Closing
    echo
    echo -e "${BOLD}${GREEN}✨ Demo complete!${RESET}"
    echo
    echo -e "${DIM}Learn more: https://github.com/usemantle/dugout${RESET}"
    sleep 2
}

# Record mode (using script command)
record() {
    local output_file="${1:-demo.cast}"
    
    echo "🎬 Recording demo to: $output_file"
    echo "   (This will take ~30 seconds)"
    echo
    
    # Use script command to record
    RECORD_MODE=true script -q -c "bash $0 run" "$output_file"
    
    echo
    echo "✅ Recording saved to: $output_file"
    echo
    echo "To convert to GIF, you can use:"
    echo "  - asciinema upload $output_file (requires asciinema)"
    echo "  - svg-term < $output_file > demo.svg (requires svg-term)"
    echo "  - Or manually convert with ttygif/termtosvg"
}

# Usage
usage() {
    cat <<EOF
Dugout Demo Script

Usage:
  $0 [command]

Commands:
  run       Run the demo (default)
  record    Record the demo using script command
  help      Show this help message

Environment Variables:
  DUGOUT_BIN       Path to dugout binary (default: ./target/debug/dugout)
  TYPING_SPEED     Seconds per character (default: 0.05)
  COMMAND_DELAY    Delay before running command (default: 1.5)
  OUTPUT_DELAY     Delay after output (default: 1.0)

Examples:
  # Just run the demo
  $0 run

  # Record to a file
  $0 record demo.cast

  # Run with custom binary
  DUGOUT_BIN=/usr/local/bin/dugout $0 run

  # Faster demo (no typing simulation)
  TYPING_SPEED=0 COMMAND_DELAY=0.5 $0 run
EOF
}

# Parse arguments
case "${1:-run}" in
    run)
        main
        ;;
    record)
        record "${2:-demo.cast}"
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo "Unknown command: $1"
        usage
        exit 1
        ;;
esac
