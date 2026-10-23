#!/usr/bin/env bash
# Runs the same checks as .github/workflows/ci.yml on this machine.
#   scripts/ci-local.sh          lint + type check + clippy + tests (a few minutes)
#   scripts/ci-local.sh --full   also `yarn tauri build` like the Test jobs (slow)
# Only covers the OS you run it on; CI also tests Windows and Linux.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
export RUSTFLAGS="-A dead_code"   # same as CI
export CARGO_TERM_COLOR=always
# A global CC/CXX (e.g. zig cc) breaks crates like `ring`; use the system compiler.
export CC="${CI_CC:-cc}" CXX="${CI_CXX:-c++}"

step() { printf '\n\033[1;34m== %s\033[0m\n' "$1"; }

step "No Apple Team ID committed"
# iOS builds take the team from APPLE_DEVELOPMENT_TEAM; a real ID in a tracked file is a leak.
if git grep -nE 'DEVELOPMENT_TEAM *[:=] *[A-Z0-9]{10}\b' -- . ':!node_modules' | grep -v 'YOUR_TEAM_ID'; then
  echo "A real Apple Team ID is in a tracked file (above). Use APPLE_DEVELOPMENT_TEAM instead." >&2
  exit 1
fi

step "yarn install --frozen-lockfile"
yarn install --frozen-lockfile

step "TypeScript"
yarn tsc --noEmit

step "Rust formatting"
(cd src-tauri && cargo fmt --all -- --check)

step "Frontend build"
# CI runs `yarn build`; its postbuild step only creates macOS installers.
yarn vite build

step "Rust clippy (all targets, all features)"
(cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings)

step "Rust tests"
(cd src-tauri && cargo test)

step "Code server tests"
(cd server && node --test)

if [[ "${1:-}" == "--full" ]]; then
  step "Tauri build"
  yarn tauri build --verbose
fi

printf '\n\033[1;32mAll local CI steps passed.\033[0m\n'
