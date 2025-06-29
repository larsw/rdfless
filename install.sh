#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# GitHub repository details
REPO="larsw/rdfless"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="rdfless"

echo -e "${BLUE}rdfless installer${NC}"
echo "================================"

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Function to get the latest release
get_latest_release() {
    curl -s "https://api.github.com/repos/$REPO/releases/latest" | 
    grep '"tag_name":' | 
    sed -E 's/.*"([^"]+)".*/\1/'
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required tools
echo "Checking for required tools..."
for tool in curl; do
    if ! command_exists "$tool"; then
        echo -e "${RED}Error: $tool is required but not installed.${NC}"
        exit 1
    fi
done

# Get the latest release version
echo "Fetching latest release information..."
LATEST_VERSION=$(get_latest_release)

if [ -z "$LATEST_VERSION" ]; then
    echo -e "${RED}Error: Could not fetch the latest release version.${NC}"
    exit 1
fi

echo -e "Latest version: ${GREEN}$LATEST_VERSION${NC}"

# Construct download URL
VERSION_NUMBER=${LATEST_VERSION#v}  # Remove 'v' prefix
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/rdfless-$VERSION_NUMBER-linux-x86_64"

echo "Downloading from: $DOWNLOAD_URL"

# Download the binary
TEMP_FILE=$(mktemp)
if curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
    echo -e "${GREEN}Download successful!${NC}"
else
    echo -e "${RED}Error: Failed to download the binary.${NC}"
    rm -f "$TEMP_FILE"
    exit 1
fi

# Install the binary
chmod +x "$TEMP_FILE"
mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"

echo -e "${GREEN}rdfless installed successfully to $INSTALL_DIR/$BINARY_NAME${NC}"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo -e "${YELLOW}Warning: $INSTALL_DIR is not in your PATH.${NC}"
    echo "To use rdfless from anywhere, you need to add it to your PATH."
    echo ""
    read -p "Would you like to add $INSTALL_DIR to your PATH? (y/N): " -r
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Determine which shell config file to update
        SHELL_CONFIG=""
        if [ -n "$BASH_VERSION" ]; then
            if [ -f "$HOME/.bashrc" ]; then
                SHELL_CONFIG="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                SHELL_CONFIG="$HOME/.bash_profile"
            fi
        elif [ -n "$ZSH_VERSION" ]; then
            SHELL_CONFIG="$HOME/.zshrc"
        elif [ "$SHELL" = "/bin/bash" ]; then
            if [ -f "$HOME/.bashrc" ]; then
                SHELL_CONFIG="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                SHELL_CONFIG="$HOME/.bash_profile"
            fi
        elif [ "$SHELL" = "/bin/zsh" ] || [ "$SHELL" = "/usr/bin/zsh" ]; then
            SHELL_CONFIG="$HOME/.zshrc"
        fi
        
        if [ -n "$SHELL_CONFIG" ]; then
            echo "" >> "$SHELL_CONFIG"
            echo "# Added by rdfless installer" >> "$SHELL_CONFIG"
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
            echo -e "${GREEN}Added $INSTALL_DIR to PATH in $SHELL_CONFIG${NC}"
            echo -e "${YELLOW}Please restart your terminal or run: source $SHELL_CONFIG${NC}"
        else
            echo -e "${YELLOW}Could not determine your shell configuration file.${NC}"
            echo "Please manually add the following to your shell's configuration file:"
            echo "export PATH=\"$INSTALL_DIR:\$PATH\""
        fi
    else
        echo "You can manually add $INSTALL_DIR to your PATH by adding this line to your shell's config file:"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
else
    echo -e "${GREEN}$INSTALL_DIR is already in your PATH.${NC}"
fi

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo "You can now run: $BINARY_NAME --help"

# Test the installation
if command_exists "$BINARY_NAME" || [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo ""
    echo "Testing installation..."
    if command_exists "$BINARY_NAME"; then
        $BINARY_NAME --version
    else
        "$INSTALL_DIR/$BINARY_NAME" --version
    fi
else
    echo -e "${YELLOW}Note: You may need to restart your terminal or update your PATH to use rdfless.${NC}"
fi
