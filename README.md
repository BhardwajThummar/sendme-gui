# SendMe GUI - Secure File Transfer

[![CI](https://github.com/BhardwajThummar/sendme-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/BhardwajThummar/sendme-gui/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/BhardwajThummar/sendme-gui)](https://github.com/BhardwajThummar/sendme-gui/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A modern, secure peer-to-peer file transfer application built with Tauri v2, React, and TypeScript.

## Features

- 🔒 **Secure P2P Transfer**: Direct file sharing using Iroh protocol
- 📱 **Cross-Platform**: Desktop (macOS, Windows, Linux) and Android support
- ⚡ **Fast & Modern**: Built with Tauri v2 and React
- 🎨 **Beautiful UI**: Clean interface with Tailwind CSS
- 🌐 **No Cloud Required**: Direct device-to-device transfers
- 📊 **Transfer Status**: Real-time progress tracking

## How sharing works

Files are transferred directly between devices using the [Iroh](https://iroh.computer/)
protocol — there's no cloud storage involved. To hand off the connection
details (a "blob ticket") to the receiving device, this app supports two
modes:

- **No server configured (default)**: the sender gets the raw ticket as
  text and a QR code; the receiver pastes it or scans it in-app.
- **Server configured** (`BASE_URL` env var): the sender instead gets a
  short numeric code, exchanged via a small backend that maps codes to
  tickets. This is optional and not required to use the app — see
  [docs/SERVER_API.md](docs/SERVER_API.md) if you want to self-host one.

## Quick Start

### Prerequisites

- Node.js 18+ and Yarn
- Rust 1.70+
- Tauri CLI

### Development

```bash
# Install dependencies
yarn install

# Run desktop app
yarn tauri dev

# Run Android app
yarn android:dev
```

### Building

```bash
# Build desktop app
yarn build

# Build Android app
yarn android:build
```

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
├── scripts/               # Build and utility scripts
│   └── version.sh         # Version management script
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
| `yarn version:check` | Check current version |
| `yarn version:patch` | Bump patch version |
| `yarn version:minor` | Bump minor version |
| `yarn version:major` | Bump major version |

## Documentation

- [VERSIONING.md](VERSIONING.md) - Version management guide
- [CHANGELOG.md](CHANGELOG.md) - Release notes
- [.github/RELEASE.md](.github/RELEASE.md) - Release process guide
- [.env.example](.env.example) - Environment configuration

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, coding standards, and
the PR process.

## License

See LICENSE file for details.

## Resources

- [Tauri Documentation](https://tauri.app/)
- [React Documentation](https://react.dev/)
- [Iroh Protocol](https://iroh.computer/)