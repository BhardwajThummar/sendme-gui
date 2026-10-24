# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Old sender work folders and copies of picked folders are removed at startup; on a
  phone they used to pile up (gigabytes) because the app is never closed cleanly
- Sharing a folder no longer fails because one file in it can't be read (a dangling
  link, a file removed meanwhile): that file is skipped and the rest is shared
- A failed share now shows the real reason instead of "Failed to send files"

### Added

- iOS support: file and folder picking, choosing where received files are saved
- Android and iOS: pick where received files are saved; overwrite / keep both
  prompt when a file already exists
- Short codes are 8 characters; the app retries a failed receive up to 3 times
- `scripts/doctor.sh` checks the toolchain; `server/` is a free, optional
  Cloudflare Worker for short codes

### Changed

- Short codes are only offered when the code server is reachable; tickets and
  QR codes always work, and sending no longer fails when the server is down
- The Download Complete screen shows what was actually received

- Plain-language error messages (unknown code, code server down, can't reach
  the sender, disk full, bad folder) instead of raw network errors
- The code server rate-limits per IP, and has automated tests
- iOS: the Apple Team ID comes from `APPLE_DEVELOPMENT_TEAM` and the generated
  Xcode project is no longer tracked, so a Team ID can't be committed by
  accident (`scripts/ci-local.sh` checks this)
- README rewritten: shorter, with screenshots

- The receive screen says whether the transfer is direct or going through a relay
  (relayed transfers are slower), as upstream sendme does
- The code-server client has a timeout (`API_TIMEOUT_SECS`, now 10 s) and the
  HTTP stack is current: `reqwest 0.13` on `ring`, which removes a second,
  older copy of rustls, hyper and h2 from the build
- Debug builds optimise dependencies, so hashing a large file is about ten times
  faster during development

- iOS: the QR scanner works. It draws the camera frames onto a canvas and decodes
  them in JavaScript, like Android; the native scanner plugin's preview never
  appeared and iOS 27's web view does not paint a live `<video>`
- The Copy Code button now says "Copied" (the message it used to show was hidden)

- A download that goes quiet for 30 seconds (for example the phone locked and iOS
  froze the app) is now treated as dead and retried from where it stopped, up to 5
  attempts. Before, it stayed on "Receiving..." forever because only errors
  triggered a retry

- The code-server client uses bundled root certificates; reqwest's default (the
  platform trust store) panicked on Android

### Removed

- Obsolete helper scripts (`fix-ios-build.sh`, `fix-cargo-permissions.sh`,
  `build-all.sh`)

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
