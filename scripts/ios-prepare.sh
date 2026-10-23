#!/usr/bin/env bash
# Generates the Xcode project from src-tauri/gen/apple/project.yml, if it isn't there.
#
# The generated project.pbxproj is not committed (it embeds your Apple Team ID),
# but `tauri ios` needs it to exist before it starts, so a fresh clone needs this
# first. `yarn ios:dev` and `yarn ios:build` run it for you. Needs `xcodegen`
# (brew install xcodegen).
set -euo pipefail
cd "$(dirname "$0")/../src-tauri/gen/apple"
[[ -f sendme-gui.xcodeproj/project.pbxproj ]] && exit 0
command -v xcodegen >/dev/null || { echo "xcodegen is missing: brew install xcodegen" >&2; exit 1; }
mkdir -p Externals assets   # build output folders the project refers to (Tauri fills them)
xcodegen generate --spec project.yml >/dev/null
# xcodegen also rewrites two tracked files with trivially different content, and
# Tauri regenerates them itself; put them back so the working tree stays clean.
git checkout -q -- sendme-gui.xcodeproj/xcshareddata sendme-gui_iOS/Info.plist 2>/dev/null || true
