#!/usr/bin/env bash
# Generates the Xcode project from src-tauri/gen/apple/project.yml.
#
# The generated project.pbxproj is not committed (it embeds your Apple Team ID),
# but `tauri ios` needs it to exist before it starts, so a fresh clone needs this
# first. `yarn ios:dev` and `yarn ios:build` run it for you. Needs `xcodegen`
# (brew install xcodegen).
set -euo pipefail
cd "$(dirname "$0")/../src-tauri/gen/apple"
command -v xcodegen >/dev/null || { echo "xcodegen is missing: brew install xcodegen" >&2; exit 1; }
mkdir -p Externals assets   # build output folders the project refers to (Tauri fills them)
xcodegen generate --spec project.yml >/dev/null
