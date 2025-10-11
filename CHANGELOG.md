# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-10-11

### Added

- Industry-standard versioning system with automated version management
- Version display in application UI
- Comprehensive changelog tracking

### Changed

- Enhanced Android file transfer with async/await
- Refactored logging system to use centralized logger utility
- Improved file sharing and receiving components for Android support

### Fixed

- Android compatibility issues with file transfers

## [0.1.0] - 2025-01-01

### Added

- Initial release of SendMe GUI Tauri application
- Secure peer-to-peer file transfer functionality
- Cross-platform support (Desktop and Android)
- Modern React UI with Tailwind CSS
- Tauri v2 integration
- File sending and receiving capabilities
- Android-specific optimizations
- Centralized configuration management

### Features

- **File Transfer**: Secure P2P file transfer using Iroh protocol
- **Cross-Platform**: Desktop (macOS, Windows, Linux) and Android support
- **Modern UI**: Clean, responsive interface built with React and Tailwind CSS
- **Configuration**: Environment-based configuration management
- **Logging**: Comprehensive logging system for debugging

---

## Version Guidelines

### Version Format

We follow [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version (X.0.0): Incompatible API changes
- **MINOR** version (0.X.0): New features (backward compatible)
- **PATCH** version (0.0.X): Bug fixes (backward compatible)

### Change Categories

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### How to Update

Use the version management script:

```bash
# Bump patch version (0.1.0 -> 0.1.1)
./scripts/version.sh patch

# Bump minor version (0.1.0 -> 0.2.0)
./scripts/version.sh minor

# Bump major version (0.1.0 -> 1.0.0)
./scripts/version.sh major

# Set specific version
./scripts/version.sh 1.2.3
```

[Unreleased]: https://github.com/yourusername/sendme-gui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/sendme-gui/releases/tag/v0.1.0
