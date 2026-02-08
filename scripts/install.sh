#!/bin/sh
set -eu

REPO="usealtoal/burrow"
INSTALL_DIR="${BURROW_INSTALL_DIR:-$HOME/.burrow/bin}"

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

    if [ -n "${BURROW_VERSION:-}" ]; then
        version="$BURROW_VERSION"
    else
        version=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
            | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/')
    fi

    url="https://github.com/$REPO/releases/download/v${version}/burrow-${target}.tar.gz"

    echo "downloading burrow v${version} for ${target}..."
    tmpdir=$(mktemp -d)
    curl -fsSL "$url" | tar xz -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/burrow" "$INSTALL_DIR/burrow"
    chmod +x "$INSTALL_DIR/burrow"
    rm -rf "$tmpdir"

    echo "installed burrow to $INSTALL_DIR/burrow"

    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) echo "add $INSTALL_DIR to your PATH" ;;
    esac
}

main
