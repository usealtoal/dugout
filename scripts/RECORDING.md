# Recording a Dugout Demo GIF

Quick guide for creating a professional demo GIF for the README.

## Recommended: asciinema + agg

This produces the cleanest, most professional results.

### 1. Install asciinema

```bash
# Ubuntu/Debian
sudo apt-get update && sudo apt-get install asciinema

# macOS
brew install asciinema

# Or via pip
pip install asciinema
```

### 2. Install agg (asciinema GIF generator)

```bash
cargo install --git https://github.com/asciinema/agg
```

### 3. Record the demo

```bash
cd /path/to/dugout

# Record with asciinema
asciinema rec -c "./scripts/demo.sh run" demo.cast

# This will run the demo and save to demo.cast
```

### 4. Convert to GIF

```bash
# Basic conversion
agg demo.cast demo.gif

# Customized (recommended settings)
agg \
  --cols 100 \
  --rows 30 \
  --speed 1.2 \
  --theme monokai \
  --font-size 14 \
  demo.cast assets/demo.gif
```

### 5. Optimize the GIF (optional)

```bash
# Install gifsicle
sudo apt-get install gifsicle  # or brew install gifsicle

# Optimize
gifsicle -O3 --colors 256 assets/demo.gif -o assets/demo-optimized.gif
```

## Alternative: svg-term (for SVG)

SVG animations are smaller and scale better, but not all platforms support them well in READMEs.

```bash
# Install
npm install -g svg-term-cli

# Record
asciinema rec -c "./scripts/demo.sh run" demo.cast

# Convert
svg-term --in demo.cast --out assets/demo.svg \
  --window \
  --width 80 \
  --height 24 \
  --term iterm2
```

## Quick Recording Tips

### Before Recording

1. **Set terminal size:**
   ```bash
   # Resize to 100x30 (cols x rows)
   printf '\e[8;30;100t'
   ```

2. **Clear terminal:**
   ```bash
   clear && reset
   ```

3. **Test the demo first:**
   ```bash
   TYPING_SPEED=0 ./scripts/demo.sh run
   ```

### During Recording

- Let the demo run without interruption
- The script handles all timing automatically
- Default demo is ~30 seconds (perfect for README)

### After Recording

- Review the .cast file: `asciinema play demo.cast`
- Adjust speed in conversion: `--speed 1.5` makes it faster
- Test the GIF size: aim for < 5MB

## Custom Timing

For a slower, more dramatic demo:

```bash
asciinema rec -c "TYPING_SPEED=0.08 COMMAND_DELAY=2 OUTPUT_DELAY=1.5 ./scripts/demo.sh run" demo-slow.cast
```

For a fast, concise demo:

```bash
asciinema rec -c "TYPING_SPEED=0 COMMAND_DELAY=0.5 OUTPUT_DELAY=0.3 ./scripts/demo.sh run" demo-fast.cast
```

## agg Options Reference

```bash
agg [OPTIONS] <INPUT> <OUTPUT>

Key options:
  --cols <COLS>              Terminal width (default: 80)
  --rows <ROWS>              Terminal height (default: 24)
  --speed <SPEED>            Playback speed multiplier (default: 1.0)
  --theme <THEME>            Color theme: asciinema, monokai, dracula, etc.
  --font-size <SIZE>         Font size in pixels (default: 14)
  --font-family <FAMILY>     Font family (default: "JetBrains Mono, Courier New")
  --fps-cap <FPS>            Max FPS (default: 30)
  --last-frame-duration <S>  Freeze last frame (seconds, default: 1)
```

Popular themes:
- `monokai` — Dark, professional (recommended)
- `dracula` — Popular dark theme
- `solarized-dark` / `solarized-light`
- `gruvbox-dark` / `gruvbox-light`

## Full Example Workflow

```bash
#!/bin/bash
# Complete demo recording workflow

set -e

cd /path/to/dugout

# 1. Test the demo
echo "Testing demo..."
TYPING_SPEED=0 COMMAND_DELAY=0.3 ./scripts/demo.sh run

# 2. Record with asciinema
echo "Recording..."
asciinema rec -c "./scripts/demo.sh run" demo.cast

# 3. Preview
echo "Previewing recording..."
asciinema play demo.cast

# 4. Convert to GIF
echo "Converting to GIF..."
agg \
  --cols 100 \
  --rows 30 \
  --speed 1.2 \
  --theme monokai \
  --font-size 14 \
  --last-frame-duration 2 \
  demo.cast assets/demo.gif

# 5. Optimize
echo "Optimizing GIF..."
gifsicle -O3 --colors 256 assets/demo.gif -o assets/demo-final.gif

# 6. Check file size
echo "Final size:"
ls -lh assets/demo-final.gif

echo "✅ Done! Use assets/demo-final.gif in your README"
```

## Troubleshooting

**GIF is too large (> 5MB):**
- Reduce FPS: `--fps-cap 15`
- Reduce colors: `gifsicle --colors 128`
- Reduce size: `--cols 80 --rows 24`
- Speed it up: `--speed 1.5`

**Text is blurry:**
- Increase font size: `--font-size 16`
- Use a better font: `--font-family "JetBrains Mono"`

**Colors look wrong:**
- Try different themes: `--theme dracula`
- Check terminal theme matches recording

**Timing is off:**
- Adjust speed: `--speed 1.2` (faster) or `--speed 0.8` (slower)
- Re-record with different delays in the script

## Resources

- [asciinema](https://asciinema.org/) — Terminal recording tool
- [agg](https://github.com/asciinema/agg) — GIF generator
- [svg-term](https://github.com/marionebl/svg-term-cli) — SVG generator
- [gifsicle](https://www.lcdf.org/gifsicle/) — GIF optimizer
