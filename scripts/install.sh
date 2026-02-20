#!/bin/sh
set -eu

REPO="usemantle/dugout"
INSTALL_DIR="${DUGOUT_INSTALL_DIR:-$HOME/.dugout/bin}"
NO_MODIFY_PATH="${DUGOUT_NO_MODIFY_PATH:-}"

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux) os="unknown-linux-musl" ;;
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

    # Add to PATH if not already present
    if [ -z "$NO_MODIFY_PATH" ]; then
        case ":$PATH:" in
            *":$INSTALL_DIR:"*) ;;
            *)
                shell_config=""
                case "${SHELL:-}" in
                    */zsh) shell_config="$HOME/.zshrc" ;;
                    */bash)
                        # Prefer .bashrc, fall back to .bash_profile
                        if [ -f "$HOME/.bashrc" ]; then
                            shell_config="$HOME/.bashrc"
                        else
                            shell_config="$HOME/.bash_profile"
                        fi
                        ;;
                    */fish) shell_config="$HOME/.config/fish/config.fish" ;;
                esac

                if [ -n "$shell_config" ]; then
                    # Only add if not already in the file
                    if ! grep -q "/.dugout/bin" "$shell_config" 2>/dev/null; then
                        mkdir -p "$(dirname "$shell_config")"
                        case "${SHELL:-}" in
                            */fish)
                                printf '\n# Added by dugout installer\nfish_add_path "$HOME/.dugout/bin"\n' >> "$shell_config"
                                ;;
                            *)
                                printf '\n# Added by dugout installer\nexport PATH="$HOME/.dugout/bin:$PATH"\n' >> "$shell_config"
                                ;;
                        esac
                        echo "added $INSTALL_DIR to PATH in $shell_config"
                    fi
                    # Update current session
                    export PATH="$INSTALL_DIR:$PATH"
                else
                    echo "add $INSTALL_DIR to your PATH"
                fi
                ;;
        esac
    fi

    echo "run 'dugout setup' to get started"
}

main
