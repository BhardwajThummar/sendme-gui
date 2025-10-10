# Versioning Guide

This document describes the industry-standard versioning system implemented in SendMe GUI.

## Overview

This project follows [Semantic Versioning 2.0.0](https://semver.org/) (SemVer) standards:

```
MAJOR.MINOR.PATCH

Example: 1.2.3
  │   │   └─ PATCH: Bug fixes and minor changes
  │   └───── MINOR: New features (backward compatible)
  └───────── MAJOR: Breaking changes (not backward compatible)
```

## Quick Start

### Check Current Version
```bash
yarn version:check
# Output: v0.1.0
```

### Bump Version
```bash
# Bump patch version (0.1.0 -> 0.1.1)
yarn version:patch

# Bump minor version (0.1.0 -> 0.2.0)
yarn version:minor

# Bump major version (0.1.0 -> 1.0.0)
yarn version:major

# Set specific version
yarn version 1.5.0
```

## Version Management Script

The [scripts/version.sh](scripts/version.sh) script automates version management:

### Features
- ✅ Syncs version across all files (package.json, Cargo.toml, tauri.conf.json)
- ✅ Updates CHANGELOG.md automatically
- ✅ Creates git commits and tags
- ✅ Follows semantic versioning standards
- ✅ Includes dry-run mode for safety
- ✅ Color-coded terminal output

### Usage Examples

**Basic version bump:**
```bash
./scripts/version.sh patch
```

**Preview changes without applying:**
```bash
./scripts/version.sh minor --dry-run
```

**Skip git operations:**
```bash
./scripts/version.sh patch --no-git
```

**Set specific version:**
```bash
./scripts/version.sh 2.0.0
```

### Script Options

| Option | Description |
|--------|-------------|
| `patch` | Bump patch version (1.2.3 → 1.2.4) |
| `minor` | Bump minor version (1.2.3 → 1.3.0) |
| `major` | Bump major version (1.2.3 → 2.0.0) |
| `X.Y.Z` | Set specific version |
| `--dry-run` | Preview changes without applying |
| `--no-git` | Skip git commit and tag creation |
| `-h, --help` | Show help message |

## Files Synchronized

The version management system automatically updates these files:

1. **[package.json](package.json)** - Frontend package version
2. **[src-tauri/Cargo.toml](src-tauri/Cargo.toml)** - Rust package version
3. **[src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)** - Tauri app version
4. **[CHANGELOG.md](CHANGELOG.md)** - Release notes

## Changelog Management

### Structure
The [CHANGELOG.md](CHANGELOG.md) follows the [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [Unreleased]
### Added
- New features go here

### Changed
- Changes to existing functionality

### Fixed
- Bug fixes

## [1.2.0] - 2025-01-15
### Added
- Feature X
- Feature Y
```

### Change Categories
- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### Workflow
1. As you develop, add changes under `[Unreleased]` section
2. When ready to release, run the version script
3. Script automatically converts `[Unreleased]` to `[X.Y.Z] - DATE`

## Release Workflow

### 1. Update Changelog
Before bumping version, ensure CHANGELOG.md has all changes listed under `[Unreleased]`:

```markdown
## [Unreleased]
### Added
- New file transfer progress indicator
- Android notification support

### Fixed
- Connection timeout on slow networks
```

### 2. Bump Version
Choose the appropriate version bump:

```bash
# For bug fixes
yarn version:patch

# For new features
yarn version:minor

# For breaking changes
yarn version:major
```

The script will:
- Update all version files
- Update CHANGELOG.md (convert Unreleased → version)
- Create a git commit: `chore(release): bump version to X.Y.Z`
- Create a git tag: `vX.Y.Z`

### 3. Review Changes
```bash
git log -1 --stat
git show vX.Y.Z
```

### 4. Build and Test
```bash
# Build the application
yarn build

# Test on desktop
yarn tauri dev

# Test on Android
yarn android:build
```

### 5. Push to Remote
```bash
# Push commit and tags
git push origin tauri-v2
git push origin vX.Y.Z

# Or push all tags at once
git push origin tauri-v2 --tags
```

### 6. Create GitHub Release
After pushing tags, create a GitHub release:

1. Go to GitHub repository → Releases → Draft new release
2. Select the tag you just pushed (e.g., `v1.2.0`)
3. Copy the changelog entries for this version
4. Attach build artifacts (if applicable)
5. Publish release

## Version Display in UI

The application displays the current version in the footer.

### Implementation
Located in [src/utils/version.ts](src/utils/version.ts):

```typescript
import { getVersionString } from './utils/version';

// Returns "v1.2.3"
const version = getVersionString();
```

### Available Utilities

```typescript
import VERSION from './utils/version';

// Get full version string
VERSION.full; // "1.2.3"

// Get display version
VERSION.display; // "v1.2.3"

// Get version parts
VERSION.parts; // { major: 1, minor: 2, patch: 3 }

// Check if development version
VERSION.isDev; // true if version starts with 0.

// Get full version info
VERSION.getFullInfo();
// {
//   version: "1.2.3",
//   name: "sendme-gui-tauri-1",
//   isDevelopment: false,
//   buildDate: "2025-01-15T10:30:00.000Z"
// }
```

## Best Practices

### When to Bump Versions

#### Patch (X.Y.1)
- Bug fixes
- Performance improvements
- Documentation updates
- Internal refactoring
- Security patches

#### Minor (X.1.0)
- New features (backward compatible)
- New API endpoints
- UI enhancements
- Deprecations

#### Major (1.0.0)
- Breaking changes
- Removed features
- Incompatible API changes
- Major redesigns

### Pre-Release Versions
For beta/alpha releases, use version format with suffix:
```
1.0.0-alpha.1
1.0.0-beta.1
1.0.0-rc.1
```

To set pre-release version:
```bash
./scripts/version.sh 1.0.0-beta.1
```

### Version 0.x.x
- Versions starting with `0.` indicate early development
- Breaking changes can occur in minor versions
- Use `0.x.x` until the API is stable

## Git Tags

### Format
All version tags follow the format: `vX.Y.Z`

Examples:
- `v1.0.0`
- `v1.2.3`
- `v2.0.0-beta.1`

### List Tags
```bash
# List all tags
git tag -l

# List tags with messages
git tag -l -n
```

### Delete Tags
```bash
# Delete local tag
git tag -d v1.2.3

# Delete remote tag
git push origin :refs/tags/v1.2.3
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Get version from tag
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_ENV

      - name: Build application
        run: yarn build

      - name: Create release
        uses: actions/create-release@v1
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ env.VERSION }}
```

## Troubleshooting

### Version Mismatch
If versions are out of sync across files:

```bash
# Manually set version to sync all files
./scripts/version.sh 1.2.3
```

### Uncommitted Changes
The script requires a clean working directory:

```bash
# Check status
git status

# Commit or stash changes
git add .
git commit -m "feat: your changes"
```

### Failed Tag Creation
If tag already exists:

```bash
# Delete existing tag
git tag -d v1.2.3
git push origin :refs/tags/v1.2.3

# Re-run version script
./scripts/version.sh 1.2.3
```

## Resources

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Git Tagging](https://git-scm.com/book/en/v2/Git-Basics-Tagging)

## Questions?

For questions or issues with the versioning system:
1. Check this documentation
2. Review [CHANGELOG.md](CHANGELOG.md)
3. Check [scripts/version.sh](scripts/version.sh) script
4. Open an issue on GitHub
