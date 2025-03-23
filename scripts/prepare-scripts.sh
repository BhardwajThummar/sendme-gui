# Create this file as scripts/prepare-scripts.sh
#!/bin/bash

echo "Preparing build scripts for DMG packaging..."

# Make sure all scripts are executable
chmod +x scripts/*.sh

# Copy the scripts to the appropriate locations in the build directory
mkdir -p src-tauri/target/scripts
cp scripts/*.sh src-tauri/target/scripts/