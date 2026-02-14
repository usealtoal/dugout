# Quick Start: Creating a Demo GIF

Three ways to create a professional demo GIF for Dugout:

## 🚀 Option 1: Automated (Recommended)

**One command to rule them all:**

```bash
./scripts/make-demo-gif.sh
```

This script:
- ✓ Checks for all dependencies
- ✓ Records the demo with asciinema
- ✓ Converts to GIF with agg (or SVG with svg-term)
- ✓ Optimizes the output with gifsicle
- ✓ Tells you exactly what to put in the README

**Prerequisites:**
```bash
# Install asciinema
sudo apt-get install asciinema  # or: brew install asciinema

# Install agg (recommended)
cargo install --git https://github.com/asciinema/agg

# Optional: Install gifsicle for optimization
sudo apt-get install gifsicle  # or: brew install gifsicle
```

**Output:** `assets/demo.gif` (or custom path)

---

## 🎬 Option 2: Manual Recording

**More control, step-by-step:**

```bash
# 1. Test the demo first
TYPING_SPEED=0 ./scripts/demo.sh run

# 2. Record with asciinema
asciinema rec -c "./scripts/demo.sh run" demo.cast

# 3. Convert to GIF
agg --cols 100 --rows 30 --speed 1.2 --theme monokai demo.cast assets/demo.gif

# 4. Optimize (optional)
gifsicle -O3 --colors 256 assets/demo.gif -o assets/demo-final.gif
```

---

## 📝 Option 3: Built-in Recording

**No asciinema, just bash:**

```bash
# Record using built-in script command
./scripts/demo.sh record demo.cast

# Convert later (requires manual tools)
```

Note: Output from `script` command is less clean than asciinema, so Option 1 or 2 are preferred.

---

## 🎨 Customization

### Timing

```bash
# Fast demo (~15 seconds)
TYPING_SPEED=0 COMMAND_DELAY=0.3 OUTPUT_DELAY=0.2 ./scripts/demo.sh run

# Dramatic demo (~45 seconds)
TYPING_SPEED=0.1 COMMAND_DELAY=2.5 OUTPUT_DELAY=2.0 ./scripts/demo.sh run
```

### GIF Quality

```bash
# High quality, larger file
agg --cols 120 --rows 35 --font-size 16 --theme dracula demo.cast demo.gif

# Smaller file, faster
agg --cols 80 --rows 24 --font-size 12 --speed 1.5 --fps-cap 15 demo.cast demo.gif
```

---

## 📏 Recommended Settings

For the README demo GIF:

- **Size:** 100 cols × 30 rows (good balance)
- **Speed:** 1.2× (keeps it snappy)
- **Theme:** monokai (professional, easy to read)
- **Duration:** ~25-30 seconds
- **File size:** < 5MB (GitHub friendly)

---

## 🐛 Troubleshooting

**"asciinema not found"**
```bash
sudo apt-get update && sudo apt-get install asciinema
```

**"agg not found"**
```bash
cargo install --git https://github.com/asciinema/agg
```

**"Binary not found" during demo**
```bash
# Build dugout first
cargo build --release

# Or specify path
DUGOUT_BIN=./target/release/dugout ./scripts/demo.sh run
```

**GIF is too large**
```bash
# Reduce colors
gifsicle -O3 --colors 128 input.gif -o output.gif

# Or re-encode with smaller size
agg --cols 80 --rows 24 --speed 1.5 demo.cast demo-small.gif
```

---

## 📚 Full Documentation

- [README.md](README.md) — Complete scripts documentation
- [RECORDING.md](RECORDING.md) — Detailed recording guide
- [demo.sh](demo.sh) — The demo script itself

---

## ✅ Complete Example

```bash
# Full workflow (Option 1)
cd /path/to/dugout
./scripts/make-demo-gif.sh

# The script will:
# 1. Check dependencies
# 2. Record the demo
# 3. Ask if you want to preview
# 4. Convert to GIF
# 5. Optimize the output
# 6. Show you the final result

# Output: assets/demo.gif
```

That's it! 🎉
