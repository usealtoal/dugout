# Demo Script Cheatsheet

Quick reference for creating Dugout demo GIFs.

## 🎬 Three Commands You Need

```bash
# 1. Test the demo
TYPING_SPEED=0 ./scripts/demo.sh run

# 2. Create the GIF (automated)
./scripts/make-demo-gif.sh

# 3. Done! Check assets/demo.gif
```

## 📦 Install Dependencies First

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install asciinema gifsicle
cargo install --git https://github.com/asciinema/agg

# macOS
brew install asciinema gifsicle
cargo install --git https://github.com/asciinema/agg
```

## 🎨 Common Recipes

### Fast Demo (15 seconds)
```bash
TYPING_SPEED=0 COMMAND_DELAY=0.3 OUTPUT_DELAY=0.2 \
  asciinema rec -c "./scripts/demo.sh run" demo-fast.cast
agg --speed 1.5 demo-fast.cast demo-fast.gif
```

### Perfect Demo (25 seconds, recommended)
```bash
asciinema rec -c "./scripts/demo.sh run" demo.cast
agg --cols 100 --rows 30 --speed 1.2 --theme monokai \
  --last-frame-duration 2 demo.cast assets/demo.gif
```

### Slow Demo (45 seconds, for presentations)
```bash
TYPING_SPEED=0.08 COMMAND_DELAY=2 OUTPUT_DELAY=1.5 \
  asciinema rec -c "./scripts/demo.sh run" demo-slow.cast
agg --speed 1.0 --last-frame-duration 3 demo-slow.cast demo-slow.gif
```

### Small File Size (< 2MB)
```bash
asciinema rec -c "./scripts/demo.sh run" demo.cast
agg --cols 80 --rows 24 --speed 1.5 --fps-cap 15 \
  demo.cast demo-small.gif
gifsicle -O3 --colors 128 demo-small.gif -o demo-final.gif
```

### High Quality (for website)
```bash
asciinema rec -c "./scripts/demo.sh run" demo.cast
agg --cols 120 --rows 35 --font-size 16 --theme dracula \
  --fps-cap 30 demo.cast demo-hq.gif
```

## 🎛️ Environment Variables

| Variable | Default | Use Case |
|----------|---------|----------|
| `TYPING_SPEED=0` | `0.05` | Instant (no typing animation) |
| `TYPING_SPEED=0.1` | `0.05` | Slow, dramatic |
| `COMMAND_DELAY=0.3` | `1.5` | Fast transitions |
| `COMMAND_DELAY=2.5` | `1.5` | Slow, clear |
| `OUTPUT_DELAY=0.2` | `1.0` | Fast flow |
| `DUGOUT_BIN=/path/to/bin` | auto | Custom binary |

## 🎨 agg Themes

- `monokai` — Dark, professional (recommended)
- `dracula` — Popular dark theme
- `solarized-dark` — Classic dark
- `solarized-light` — Classic light
- `gruvbox-dark` — Retro dark
- `github` — Light, clean

## 📏 Common Sizes

| Use Case | Cols | Rows | Speed | Size |
|----------|------|------|-------|------|
| README | 100 | 30 | 1.2× | ~3-5MB |
| Twitter | 80 | 24 | 1.5× | ~2-3MB |
| Blog | 120 | 35 | 1.0× | ~5-8MB |
| Docs | 100 | 30 | 1.2× | ~3-5MB |

## 🐛 Quick Fixes

**Demo too long?**
```bash
TYPING_SPEED=0 COMMAND_DELAY=0.3 ./scripts/demo.sh run
# or
agg --speed 2.0 demo.cast demo-fast.gif
```

**GIF too large?**
```bash
gifsicle -O3 --colors 128 --lossy=80 input.gif -o output.gif
```

**Text too small?**
```bash
agg --font-size 16 demo.cast demo-big.gif
```

**Colors look wrong?**
```bash
agg --theme dracula demo.cast demo-colorful.gif
```

**Binary not found?**
```bash
cargo build --release
DUGOUT_BIN=./target/release/dugout ./scripts/demo.sh run
```

## 📋 Complete Workflow

```bash
# One-liner for production README demo
asciinema rec -c "./scripts/demo.sh run" demo.cast && \
agg --cols 100 --rows 30 --speed 1.2 --theme monokai \
  --last-frame-duration 2 demo.cast assets/demo.gif && \
gifsicle -O3 --colors 256 assets/demo.gif -o assets/demo.gif && \
echo "✅ Done! Size: $(du -h assets/demo.gif | cut -f1)"
```

## 🔗 Resources

- [asciinema.org](https://asciinema.org/)
- [github.com/asciinema/agg](https://github.com/asciinema/agg)
- [github.com/kohler/gifsicle](https://github.com/kohler/gifsicle)

---

**TL;DR:** Run `./scripts/make-demo-gif.sh` and follow the prompts. Done.
