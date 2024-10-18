#!/bin/bash

set -e

# Define the GitHub repo and release
GITHUB_API="https://api.github.com/repos/brettmayson/HEMTT/releases/latest"

# Get the latest release info
RELEASE_INFO=$(curl -s "$GITHUB_API")
DOWNLOAD_URL=""

# Detect OS and architecture
case "$(uname -s)" in
    Linux*)
        ARCH="$(uname -m)"
        if [ "$ARCH" == "x86_64" ]; then
            DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o 'http[^"]*' | grep 'linux-x64' | head -n 1)
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    Darwin*)
        ARCH="$(uname -m)"
        if [ "$ARCH" == "x86_64" ]; then
            DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o 'http[^"]*' | grep 'darwin-x64' | head -n 1)
        elif [ "$ARCH" == "arm64" ]; then
            DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o 'http[^"]*' | grep 'darwin-arm64' | head -n 1)
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported OS: $(uname -s)"
        exit 1
        ;;
esac

# Download the binary
if [ -z "$DOWNLOAD_URL" ]; then
    echo "Could not find a suitable binary for your system."
    exit 1
fi

echo "Downloading from $DOWNLOAD_URL..."
mkdir -p /tmp/hemtt-installer
curl -L -o /tmp/hemtt-installer/hemtt "$DOWNLOAD_URL"

# Make it executable
chmod +x /tmp/hemtt-installer/hemtt

binaryLocation="$HOME/.local/bin"
if [ "$(uname -s)" == "Darwin" ]; then
    binaryLocation="$home/bin"
fi
mkdir -p "$binaryLocation"

if ! echo "$PATH" | grep -q "$binaryLocation"; then
    # Array of common shell configuration files
    config_files=("$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.profile")
    for config in "${config_files[@]}"; do
        # Check if the file exists
        if [ -f "$config" ]; then
            # Check if binaryLocation is already in the file
            if ! grep -q -s "export PATH=$binaryLocation:\$PATH" "$config"; then
                echo "Appending $binaryLocation to $config"
                echo "" >>"$config"
                echo "# Added by HEMTT" >>"$config"
                echo "export PATH=$binaryLocation:\$PATH" >>"$config"
            fi
        fi
    done
    # add to the current session
    export PATH=$binaryLocation:$PATH
fi

# Move the binary to the correct location, check for sudo
if [ -w "$binaryLocation" ]; then
    mv /tmp/hemtt-installer/hemtt "$binaryLocation"
else
    echo "The installer was unable to move the binary to $binaryLocation."
    exit 1
fi

echo "Installation complete. You can run HEMTT using 'hemtt'."
