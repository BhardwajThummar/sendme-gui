# SendMe GUI - Secure File Transfer

[![CI](https://github.com/BhardwajThummar/sendme-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/BhardwajThummar/sendme-gui/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/BhardwajThummar/sendme-gui)](https://github.com/BhardwajThummar/sendme-gui/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A modern, secure peer-to-peer file transfer application built with Tauri v2, React, and TypeScript.

## Features

- 🔒 **Secure P2P Transfer**: Direct file sharing using Iroh protocol
- 📱 **Cross-Platform**: Desktop (macOS, Windows, Linux), Android and iOS
- ⚡ **Fast & Modern**: Built with Tauri v2 and React
- 🎨 **Beautiful UI**: Clean interface with Tailwind CSS
- 🌐 **No Cloud Required**: Direct device-to-device transfers
- 📊 **Transfer Status**: Real-time progress tracking

## How sharing works

Files are transferred directly between devices using the [Iroh](https://iroh.computer/)
protocol, with no cloud storage involved. The sender gets a **ticket** (shown
as text and a QR code); the receiver pastes it or scans it in the app.

Optionally, a tiny server can swap the long ticket for a short 8-character
code. It is not needed: the app only offers short codes when `BASE_URL` points
at a reachable server, and tickets always work. A free, deployable one is in
[server/](server/README.md); the API is in [docs/SERVER_API.md](docs/SERVER_API.md).

## Quick Start

```bash
scripts/doctor.sh        # checks your toolchain and says what to install
yarn install
yarn tauri dev           # desktop
```

Android and iOS are optional extras that `doctor.sh` also checks:

| Platform | Needs | Run | Build |
|----------|-------|-----|-------|
| Desktop | Node 18+, Yarn, Rust | `yarn tauri dev` | `yarn build` |
| Android | Android Studio (SDK + NDK), `ANDROID_HOME`, `NDK_HOME` | `yarn android:dev` | `yarn android:build` |
| iOS (macOS only) | Xcode, `xcodegen`, an Apple Team ID | `yarn ios:dev` | `yarn ios:build` |

For iOS, replace `YOUR_TEAM_ID` (`DEVELOPMENT_TEAM`) in
`src-tauri/gen/apple/project.yml` with your own Apple Team ID locally; don't
commit it.

If the Rust build fails in the `ring` crate, a global `CC` (such as `zig cc`)
is the usual cause: run with `CC=cc CXX=c++`.

Before opening a PR, run `scripts/ci-local.sh` (see [CONTRIBUTING.md](CONTRIBUTING.md)).

## Version Management

This project follows [Semantic Versioning 2.0.0](https://semver.org/). See [VERSIONING.md](VERSIONING.md) for detailed information.

### Check Version
```bash
yarn version:check
```

### Bump Version
```bash
# Bump patch version (bug fixes)
yarn version:patch

# Bump minor version (new features)
yarn version:minor

# Bump major version (breaking changes)
yarn version:major

# Set specific version
yarn version 1.2.3
```

For more details, see [VERSIONING.md](VERSIONING.md).

## Releases

The project uses GitHub Actions to automate multiplatform builds and releases. When you push a version tag, it automatically builds for:

- **Windows** (x86_64)
- **macOS** (Intel and Apple Silicon)
- **Linux** (x86_64)
- **Android** (Universal APK)

To create a release:

```bash
# 1. Bump version
yarn version:patch  # or minor/major

# 2. Push tag to GitHub
git push origin main
git push origin v0.1.0  # Replace with your version
```

GitHub Actions will automatically build and publish the release. See [.github/RELEASE.md](.github/RELEASE.md) for the complete release process.

## Project Structure

```
.
├── src/                    # React frontend
│   ├── components/        # UI components
│   ├── context/           # React contexts
│   ├── utils/             # Utility functions
│   └── App.tsx            # Main app component
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── server/                # Optional short-code server (Cloudflare Worker)
├── scripts/               # doctor.sh, ci-local.sh, version and packaging scripts
├── CHANGELOG.md           # Release notes
├── VERSIONING.md          # Versioning guide
└── package.json           # Node dependencies
```

## Configuration

Copy [.env.example](.env.example) to `.env` and adjust as needed — all values
are optional and have sensible defaults.

## Development Tools

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Code Formatting

```bash
# Format Rust code
cargo +nightly fmt -- --unstable-features --config imports_granularity=Crate,group_imports=StdExternalCrate

# Format TypeScript/React code
yarn prettier --write .
```

## Scripts

| Command | Description |
|---------|-------------|
| `yarn dev` | Start development server |
| `yarn build` | Build production app |
| `yarn tauri` | Run Tauri CLI commands |
| `yarn android:dev` | Run on Android device/emulator |
| `yarn android:build` | Build Android APK |
| `yarn ios:dev` | Run on an iOS simulator/device (macOS) |
| `yarn ios:build` | Build the iOS app (macOS) |
| `yarn version:check` | Check current version |
| `yarn version:patch` | Bump patch version |
| `yarn version:minor` | Bump minor version |
| `yarn version:major` | Bump major version |

## Documentation

- [VERSIONING.md](VERSIONING.md) - Version management guide
- [CHANGELOG.md](CHANGELOG.md) - Release notes
- [.github/RELEASE.md](.github/RELEASE.md) - Release process guide
- [.env.example](.env.example) - Environment configuration
- [docs/SERVER_API.md](docs/SERVER_API.md) - Optional short-code server API

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, coding standards, and
the PR process.

## License

See LICENSE file for details.

## Resources

- [Tauri Documentation](https://tauri.app/)
- [React Documentation](https://react.dev/)
- [Iroh Protocol](https://iroh.computer/)