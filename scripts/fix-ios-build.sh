#!/bin/bash

# Fix iOS build issues by removing corrupted .tauri directories
# These directories are owned by root and need sudo to remove

echo "Fixing iOS build issues..."
echo "This script will remove corrupted .tauri directories from Cargo registry"
echo ""

# Remove the problematic directories
echo "Removing tauri-plugin-dialog .tauri directory..."
sudo rm -rf "$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-plugin-dialog-2.2.1/.tauri"

echo "Removing tauri-plugin-shell .tauri directory..."
sudo rm -rf "$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-plugin-shell-2.2.1/.tauri"

echo ""
echo "Done! Now you can rebuild the iOS app."
echo "Run: yarn tauri ios dev"
