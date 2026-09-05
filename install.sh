#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing Everything Linux..."

# Install binaries to ~/.local/bin
mkdir -p "$HOME/.local/bin"
cp -f "$DIR/everything-gui" "$HOME/.local/bin/"
cp -f "$DIR/everything-cli" "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/everything-gui" "$HOME/.local/bin/everything-cli"

# Install desktop shortcut
mkdir -p "$HOME/.local/share/applications"
sed "s|Exec=.*|Exec=$HOME/.local/bin/everything-gui|g" "$DIR/everything.desktop" > "$HOME/.local/share/applications/everything.desktop"
chmod +x "$HOME/.local/share/applications/everything.desktop"

# Update desktop database
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

echo "Installation complete!"
echo "Run 'everything-cli' in your terminal or launch 'Everything Linux' from your app menu."
