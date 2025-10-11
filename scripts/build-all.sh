#!/bin/bash

# Full build process including DMG creation

# Step 1: Build the Tauri app
echo "Building Tauri app..."
yarn tauri build
# or: cargo tauri build

# Step 2: Create the pkg installer
echo "Creating pkg installer..."
./create-installer.sh

# Step 3: Create the DMG
echo "Creating DMG file..."
./create-dmg.sh

echo "Build process completed!"
echo "DMG file is ready at: ./sendme-gui.dmg"