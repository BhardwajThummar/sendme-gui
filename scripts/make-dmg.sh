# Create this as scripts/make-dmg.sh
#!/bin/bash

# Build the Tauri app
yarn tauri build

# Run the installer and DMG creation scripts
bash ./scripts/create-installer.sh
bash ./scripts/create-dmg.sh

echo "DMG created successfully!"