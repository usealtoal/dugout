#!/bin/sh
set -eu

REPO="usealtoal/dugout"
INSTALL_DIR="${DUGOUT_INSTALL_DIR:-$HOME/.dugout/bin}"

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux) os="unknown-linux-gnu" ;;
        darwin) os="apple-darwin" ;;
        *) echo "error: unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "error: unsupported architecture: $arch" >&2; exit 1 ;;
    esac

    target="${arch}-${os}"

    if [ -n "${DUGOUT_VERSION:-}" ]; then
        version="$DUGOUT_VERSION"
    else
        version=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
            | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/')
    fi

    url="https://github.com/$REPO/releases/download/v${version}/dugout-${target}.tar.gz"

    echo "downloading dugout v${version} for ${target}..."
    tmpdir=$(mktemp -d)
    curl -fsSL "$url" | tar xz -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/dugout" "$INSTALL_DIR/dugout"
    chmod +x "$INSTALL_DIR/dugout"
    rm -rf "$tmpdir"

    echo "installed dugout to $INSTALL_DIR/dugout"

    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) echo "add $INSTALL_DIR to your PATH" ;;
    esac
}

main
