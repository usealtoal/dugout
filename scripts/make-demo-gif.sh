#!/usr/bin/env bash
# Automated demo GIF creation for Dugout
# Handles recording and conversion in one command

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_FILE="${1:-$PROJECT_ROOT/assets/demo.gif}"

# Colors
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
RESET='\033[0m'

# Check dependencies
check_dependency() {
    local cmd="$1"
    local install_hint="$2"
    
    if ! command -v "$cmd" &> /dev/null; then
        echo -e "${RED}✗${RESET} $cmd not found"
        echo -e "  Install: ${YELLOW}$install_hint${RESET}"
        return 1
    else
        echo -e "${GREEN}✓${RESET} $cmd found"
        return 0
    fi
}

echo -e "${BOLD}Dugout Demo GIF Creator${RESET}"
echo

echo "Checking dependencies..."
all_deps_ok=true

check_dependency "asciinema" "apt-get install asciinema  # or brew install asciinema" || all_deps_ok=false

# Check for GIF converters (try agg first, fall back to svg-term)
has_agg=false
has_svg_term=false

if command -v agg &> /dev/null; then
    echo -e "${GREEN}✓${RESET} agg found (preferred)"
    has_agg=true
elif command -v svg-term &> /dev/null; then
    echo -e "${YELLOW}⚠${RESET} svg-term found (fallback, will create SVG)"
    has_svg_term=true
    OUTPUT_FILE="${OUTPUT_FILE%.gif}.svg"
else
    echo -e "${RED}✗${RESET} No GIF/SVG converter found"
    echo -e "  Install agg: ${YELLOW}cargo install --git https://github.com/asciinema/agg${RESET}"
    echo -e "  Or svg-term: ${YELLOW}npm install -g svg-term-cli${RESET}"
    all_deps_ok=false
fi

if [ "$all_deps_ok" = false ]; then
    echo
    echo -e "${RED}Missing dependencies. Please install them and try again.${RESET}"
    exit 1
fi

echo

# Create temp directory for recording
TEMP_DIR=$(mktemp -d)
CAST_FILE="$TEMP_DIR/demo.cast"

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Step 1: Record
echo -e "${BOLD}Step 1: Recording demo...${RESET}"
echo "Output: $CAST_FILE"
echo

cd "$PROJECT_ROOT"

if ! asciinema rec -c "$SCRIPT_DIR/demo.sh run" "$CAST_FILE"; then
    echo -e "${RED}Recording failed!${RESET}"
    exit 1
fi

echo
echo -e "${GREEN}✓ Recording complete${RESET}"
echo

# Step 2: Preview (optional)
read -p "Preview recording? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    asciinema play "$CAST_FILE"
    echo
    read -p "Continue with conversion? [Y/n] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        echo "Cancelled. Recording saved at: $CAST_FILE"
        mv "$CAST_FILE" "$PROJECT_ROOT/demo.cast"
        echo "Moved to: $PROJECT_ROOT/demo.cast"
        exit 0
    fi
fi

# Step 3: Convert
echo -e "${BOLD}Step 2: Converting to ${OUTPUT_FILE##*.}...${RESET}"

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

if [ "$has_agg" = true ]; then
    # Convert with agg (to GIF)
    echo "Using agg (high quality GIF)..."
    
    agg \
        --cols 100 \
        --rows 30 \
        --speed 1.2 \
        --theme monokai \
        --font-size 14 \
        --font-family "JetBrains Mono, Consolas, monospace" \
        --last-frame-duration 2 \
        "$CAST_FILE" \
        "$OUTPUT_FILE"
    
    echo -e "${GREEN}✓ GIF created${RESET}"
    
    # Step 4: Optimize (if gifsicle is available)
    if command -v gifsicle &> /dev/null; then
        echo
        echo -e "${BOLD}Step 3: Optimizing GIF...${RESET}"
        
        TEMP_GIF="$TEMP_DIR/temp.gif"
        mv "$OUTPUT_FILE" "$TEMP_GIF"
        
        gifsicle -O3 --colors 256 "$TEMP_GIF" -o "$OUTPUT_FILE"
        
        echo -e "${GREEN}✓ GIF optimized${RESET}"
    else
        echo
        echo -e "${YELLOW}⚠ gifsicle not found, skipping optimization${RESET}"
        echo -e "  Install: ${YELLOW}apt-get install gifsicle${RESET}"
    fi
    
elif [ "$has_svg_term" = true ]; then
    # Convert with svg-term (to SVG)
    echo "Using svg-term (SVG animation)..."
    
    svg-term \
        --in "$CAST_FILE" \
        --out "$OUTPUT_FILE" \
        --window \
        --width 100 \
        --height 30 \
        --term iterm2
    
    echo -e "${GREEN}✓ SVG created${RESET}"
fi

# Step 5: Summary
echo
echo -e "${BOLD}${GREEN}✨ Done!${RESET}"
echo
echo "Output file: $OUTPUT_FILE"

if [ -f "$OUTPUT_FILE" ]; then
    file_size=$(du -h "$OUTPUT_FILE" | cut -f1)
    echo "File size: $file_size"
    
    # Warn if too large
    size_bytes=$(du -b "$OUTPUT_FILE" | cut -f1)
    if [ "$size_bytes" -gt 5242880 ]; then  # 5MB
        echo -e "${YELLOW}⚠ Warning: File is larger than 5MB${RESET}"
        echo "  Consider reducing size with:"
        echo "    --speed 1.5 (faster playback)"
        echo "    --cols 80 --rows 24 (smaller terminal)"
        echo "    gifsicle --colors 128 (fewer colors)"
    fi
fi

echo
echo "Usage in README:"
echo '```markdown'
echo "![Dugout Demo](${OUTPUT_FILE#$PROJECT_ROOT/})"
echo '```'
echo
echo "Preview:"
if [ "$has_agg" = true ] && command -v xdg-open &> /dev/null; then
    xdg-open "$OUTPUT_FILE" 2>/dev/null || true
elif [ "$has_agg" = true ] && command -v open &> /dev/null; then
    open "$OUTPUT_FILE" 2>/dev/null || true
else
    echo "  $OUTPUT_FILE"
fi
