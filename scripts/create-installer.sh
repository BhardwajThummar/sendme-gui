#!/bin/bash

# This script creates a pkg installer that can be included in your DMG
# Update paths to be relative to project root
APP_PATH="./src-tauri/target/release/bundle/macos/sendme-gui-tauri-1.app"

# Create directory structure
mkdir -p ./installer/scripts
mkdir -p ./installer/build

# Copy the app to the build folder
cp -R "./src-tauri/target/release/bundle/macos/sendme-gui-tauri-1.app" "./installer/build/"

# Copy the postinstall script
cp ./postinstall.sh ./installer/scripts/
chmod +x ./installer/scripts/postinstall.sh

# Create the pkg installer
pkgbuild --root ./installer/build \
         --scripts ./installer/scripts \
         --identifier com.sendme-gui-tauri-1.app \
         --version 0.1.0 \
         --install-location /Applications \
         ./installer/sendme-gui-tauri-1.pkg
         

echo "PKG installer created at ./installer/sendme-gui-tauri-1.pkg"