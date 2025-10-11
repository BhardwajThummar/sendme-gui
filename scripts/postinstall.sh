#!/bin/bash

# This script will run after the app is installed from the DMG

# Path to the app executable
APP_PATH="/Applications/sendme-gui.app/Contents/MacOS/sendme-gui"

# Check if the app exists
if [ ! -f "$APP_PATH" ]; then
    echo "Error: Application executable not found at $APP_PATH"
    exit 1
fi

# Set ownership to root:wheel
chown root:wheel "$APP_PATH"

# Set setuid bit
chmod u+s "$APP_PATH"

# Create and set permissions for any required directories
mkdir -p ~/.local/share/iroh
mkdir -p ~/.config/iroh
chmod -R 755 ~/.local/share/iroh
chmod -R 755 ~/.config/iroh
chown -R $USER ~/.local/share/iroh
chown -R $USER ~/.config/iroh

echo "Successfully set permissions for sendme-gui"
exit 0