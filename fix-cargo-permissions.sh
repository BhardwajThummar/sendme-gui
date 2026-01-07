#!/bin/bash

# Fix Cargo registry permissions for Tauri plugins
echo "Fixing Cargo registry permissions..."

# Change ownership of the entire plugin directories back to the user
# echo "Fixing tauri-plugin-dialog ownership..."
# sudo chown -R bhardwaj:staff /Users/bhardwaj/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-plugin-dialog-2.2.1

# echo "Fixing tauri-plugin-shell ownership..."
# sudo chown -R bhardwaj:staff /Users/bhardwaj/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-plugin-shell-2.2.1

echo "Fixing entire .cargo directory ownership..."
sudo chown -R "$(id -un)":"$(id -gn)" ~/.cargo


# to add the target ios dependencies
# rustup update
# rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios


# Set the correct SDK root for clang/bindgen
# export SDKROOT=$(xcrun --sdk iphonesimulator --show-sdk-path)
# export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=${SDKROOT}"

# final fix for sdk path
# export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(xcrun --sdk iphonesimulator --show-sdk-path)"

echo ""
echo "Done! Permissions fixed."
echo "Now run: yarn tauri ios dev"
