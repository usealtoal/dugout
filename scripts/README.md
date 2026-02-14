# Dugout Scripts

This directory contains installation and demo scripts for Dugout.

## Files

- **install.sh** — Install dugout on macOS and Linux
- **install.ps1** — Install dugout on Windows (PowerShell)
- **demo.sh** — Terminal demo/recording script for documentation

## Demo Script

The `demo.sh` script creates a professional terminal demo showing Dugout's core features. It's designed for creating GIFs, screencasts, or README examples.

### Quick Start

```bash
# Run the demo (live, ~30 seconds)
./scripts/demo.sh run

# Record the demo to a file
./scripts/demo.sh record demo.cast

# Get help
./scripts/demo.sh help
```

### What It Shows

The demo walks through a complete Dugout workflow:

1. **Initialize** — `dugout init` to create a vault
2. **Add secrets** — `dugout set` for DATABASE_URL, API_KEY, STRIPE_SECRET
3. **Retrieve** — `dugout get API_KEY` to fetch a secret
4. **List** — `dugout list` to see all keys
5. **Run** — `dugout run --` to inject secrets into a demo app

### Recording Options

#### Option 1: Built-in Recording (using `script`)

```bash
# Record to a typescript file
./scripts/demo.sh record demo.cast

# The output can be converted using various tools (see below)
```

#### Option 2: Use asciinema (recommended for GIFs)

```bash
# Install asciinema first
sudo apt-get install asciinema  # or brew install asciinema

# Record with asciinema
asciinema rec -c "./scripts/demo.sh run" demo.cast

# Upload to asciinema.org
asciinema upload demo.cast

# Or convert to GIF using agg (https://github.com/asciinema/agg)
agg demo.cast demo.gif
```

#### Option 3: Use termtosvg (for SVG output)

```bash
# Install termtosvg
pip install termtosvg

# Record
termtosvg -c "./scripts/demo.sh run" demo.svg

# Output is an animated SVG
```

#### Option 4: Use ttygif (for GIF output)

```bash
# Install ttygif
git clone https://github.com/icholy/ttygif && cd ttygif && make

# Record
ttyrec demo.tty
./scripts/demo.sh run
exit

# Convert to GIF
ttygif demo.tty
```

### Customization

Control timing and behavior with environment variables:

```bash
# Fast mode (no typing simulation)
TYPING_SPEED=0 COMMAND_DELAY=0.3 OUTPUT_DELAY=0.1 ./scripts/demo.sh run

# Slower, more dramatic (for live demos)
TYPING_SPEED=0.1 COMMAND_DELAY=2 OUTPUT_DELAY=1.5 ./scripts/demo.sh run

# Use a specific binary
DUGOUT_BIN=/usr/local/bin/dugout ./scripts/demo.sh run

# Custom demo directory (useful for debugging)
DEMO_DIR=/tmp/my-demo ./scripts/demo.sh run
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DUGOUT_BIN` | Auto-detected | Path to dugout binary |
| `TYPING_SPEED` | `0.05` | Seconds per character (0 = instant) |
| `COMMAND_DELAY` | `1.5` | Delay before running command (seconds) |
| `OUTPUT_DELAY` | `1.0` | Delay after command output (seconds) |
| `DEMO_DIR` | `/tmp/dugout-demo` | Working directory for demo |
| `RECORD_MODE` | `false` | Set to `true` for typing simulation |

### Converting Recordings

After recording, you can convert to various formats:

#### To GIF (using asciinema + agg)

```bash
# Install agg (https://github.com/asciinema/agg)
cargo install --git https://github.com/asciinema/agg

# Convert (customize size, speed, theme)
agg \
  --cols 80 \
  --rows 24 \
  --speed 1.5 \
  --theme monokai \
  demo.cast demo.gif
```

#### To SVG (using svg-term)

```bash
# Install svg-term
npm install -g svg-term-cli

# Convert
svg-term --in demo.cast --out demo.svg --window
```

#### To Video (using asciinema-player + ffmpeg)

```bash
# Use asciinema-player (browser-based)
# Then record screen or use headless Chrome with ffmpeg
# (More complex, see asciinema docs)
```

### Tips for Great Demos

1. **Keep it short** — 20-30 seconds is ideal
2. **Show the flow** — Init → Set → Get → List → Run
3. **Use realistic values** — Makes it relatable
4. **Clean output** — The script handles this automatically
5. **Test before recording** — Run `./scripts/demo.sh run` first

### Adding to README

Once you have a GIF:

```markdown
## Quick Demo

![Dugout Demo](assets/demo.gif)

See it in action: initialize a vault, add secrets, and run your app with zero config.
```

### Troubleshooting

**Binary not found:**
```bash
# Build dugout first
cargo build --release

# Or specify path
DUGOUT_BIN=./target/release/dugout ./scripts/demo.sh run
```

**Recording looks wrong:**
```bash
# Ensure terminal is 80x24 or larger
echo $COLUMNS x $LINES  # should be at least 80x24

# Try with a clean terminal
TERM=xterm-256color ./scripts/demo.sh run
```

**Timing is off:**
```bash
# Adjust delays
TYPING_SPEED=0 COMMAND_DELAY=0.5 OUTPUT_DELAY=0.3 ./scripts/demo.sh run
```

## Contributing

To improve the demo:

1. Edit `demo.sh`
2. Test with `./scripts/demo.sh run`
3. Verify recording with `./scripts/demo.sh record test.cast`
4. Submit a PR with your improvements

Keep the demo under 30 seconds and focused on core features.
