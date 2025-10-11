#!/bin/bash

# This script creates a DMG file containing the installer package
# Requires create-dmg tool: brew install create-dmg

# Create a temporary directory
TEMP_DIR=$(mktemp -d)

# Copy the pkg installer to the temp directory
cp ./installer/sendme-gui.pkg "$TEMP_DIR"

# Copy any additional files like license, readme, etc.
# cp ./LICENSE.txt "$TEMP_DIR"
# cp ./README.md "$TEMP_DIR"

# Create the DMG
create-dmg \
  --volname "Sendme Installer" \
  --volicon "./src-tauri/icons/icon.icns" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --icon "sendme-gui.pkg" 200 190 \
  --hide-extension "sendme-gui.pkg" \
  --app-drop-link 600 185 \
  "./sendme-gui.dmg" \
  "$TEMP_DIR"

# Clean up
rm -rf "$TEMP_DIR"

echo "DMG created at ./sendme-gui.dmg"