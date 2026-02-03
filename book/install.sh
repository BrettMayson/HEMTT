#!/bin/sh

set -e

GITHUB_API="https://api.github.com/repos/brettmayson/HEMTT/releases/latest"
RELEASE_INFO=$(curl -s "$GITHUB_API")
DOWNLOAD_URL=""

# --- macOS preflight checks ---
if [ "$(uname -s)" = "Darwin" ]; then
    if ! command -v brew >/dev/null 2>&1; then
        echo "Homebrew is required on macOS but was not found."
        echo "Install it from https://brew.sh and re-run this installer."
        exit 1
    fi

    if ! brew list --versions openssl@3 >/dev/null 2>&1; then
        echo "openssl@3 not found. Installing via Homebrew..."
        brew install openssl@3
    fi
fi

# --- Determine download URL ---
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        if [ "$ARCH" = "x86_64" ]; then
            DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o 'http[^"]*' | grep 'linux-x64' | head -n 1)
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    Darwin*)
        if [ "$ARCH" = "arm64" ]; then
            DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o 'http[^"]*' | grep 'darwin-arm64' | head -n 1)
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Could not find a suitable binary for your system."
    exit 1
fi

echo "Downloading from $DOWNLOAD_URL..."
TMP_DIR=$(mktemp -d)
curl -L -o "$TMP_DIR/hemtt" "$DOWNLOAD_URL"
chmod +x "$TMP_DIR/hemtt"

# --- Install location ---
BINARY_DIR="$HOME/.local/bin"
mkdir -p "$BINARY_DIR"

# --- Update PATH ---
update_path() {
    CONFIG="$1"
    if [ ! -f "$CONFIG" ]; then
        touch "$CONFIG"
    fi
    if ! grep -q "$BINARY_DIR" "$CONFIG"; then
        echo "" >> "$CONFIG"
        echo "# Added by HEMTT installer" >> "$CONFIG"
        echo "export PATH=\"$BINARY_DIR:\$PATH\"" >> "$CONFIG"
        echo "Added $BINARY_DIR to PATH in $CONFIG"
    fi
}

# --- Update shell completions ---
update_completion() {
    CONFIG="$1"
    SHELL_NAME="$2"
    if [ ! -f "$CONFIG" ]; then
        touch "$CONFIG"
    fi
    if ! grep -q "source <(hemtt manage completions" "$CONFIG"; then
        echo "" >> "$CONFIG"
        echo "# Added by HEMTT installer" >> "$CONFIG"
        echo "source <(hemtt manage completions $SHELL_NAME)" >> "$CONFIG"
        echo "Added HEMTT completions to $CONFIG"
    fi
}

# --- macOS specific ---
if [ "$OS" = "Darwin" ]; then
    # PATH for Zsh login shells
    update_path "$HOME/.zprofile"

    # Completions for Zsh
    update_completion "$HOME/.zshrc" "zsh"

    # Completions for Bash if present
    [ -f "$HOME/.bashrc" ] && update_completion "$HOME/.bashrc" "bash"

    # Completions for Fish if config exists
    FISH_CONFIG="$HOME/.config/fish/config.fish"
    if [ -f "$FISH_CONFIG" ]; then
        update_completion "$FISH_CONFIG" "fish"
    fi
fi

# --- Linux ---
if [ "$OS" = "Linux" ]; then
    CONFIG_FILES="$HOME/.bashrc $HOME/.bash_profile $HOME/.profile $HOME/.zshrc $HOME/.zprofile"
    for CONFIG in $CONFIG_FILES; do
        [ -f "$CONFIG" ] && update_path "$CONFIG"
    done

    # Fish completions
    FISH_CONFIG="$HOME/.config/fish/config.fish"
    if [ -f "$FISH_CONFIG" ]; then
        update_completion "$FISH_CONFIG" "fish"
    fi
fi

# --- Move binary ---
mv "$TMP_DIR/hemtt" "$BINARY_DIR/hemtt"
rm -rf "$TMP_DIR"

echo "Installation complete."
echo "Please restart your terminal or source the relevant shell config."
echo "You can run HEMTT using: hemtt"
