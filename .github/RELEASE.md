# Release Process

This document describes how to create and publish releases for SendMe GUI.

## Overview

The project uses GitHub Actions to automate multiplatform builds and releases. Releases are created automatically when you push a version tag to the repository.

## Supported Platforms

The automated release process builds for the following platforms:

- **Windows** (x86_64)
- **macOS** (Intel and Apple Silicon)
- **Linux** (x86_64)
- **Android** (Universal APK)

## Release Workflow

### 1. Prepare the Release

Before creating a release, ensure:

- All changes are committed and pushed to the `tauri-v2` branch
- CI tests are passing
- The changelog is up to date

### 2. Bump Version

Use the version management script to bump the version:

```bash
# For a patch release (0.1.0 -> 0.1.1)
yarn version:patch

# For a minor release (0.1.0 -> 0.2.0)
yarn version:minor

# For a major release (0.1.0 -> 1.0.0)
yarn version:major

# Or set a specific version
yarn version 1.2.3
```

This script will:
- Update version in `package.json`, `Cargo.toml`, and `tauri.conf.json`
- Update the changelog
- Create a git commit
- Create a git tag

### 3. Push the Release

Push the version commit and tag to GitHub:

```bash
git push origin tauri-v2
git push origin v0.1.0  # Replace with your version
```

### 4. Automated Build Process

When you push a tag, GitHub Actions will automatically:

1. **Create a draft release** on GitHub
2. **Build desktop applications** for all platforms in parallel:
   - Windows: `.exe` installer and `.msi` package
   - macOS (Intel): `.dmg` and `.app.tar.gz`
   - macOS (Apple Silicon): `.dmg` and `.app.tar.gz`
   - Linux: `.AppImage`, `.deb`, and `.rpm`
3. **Build Android application**: Universal APK
4. **Upload all artifacts** to the GitHub release
5. **Publish the release** (convert from draft to published)

### 5. Monitor the Build

1. Go to the **Actions** tab in your GitHub repository
2. Find the "Release" workflow run for your tag
3. Monitor the build progress for each platform
4. The entire process typically takes 20-30 minutes

### 6. Verify the Release

Once the workflow completes:

1. Go to the **Releases** page in your repository
2. Verify that all artifacts are uploaded:
   - Windows installers
   - macOS DMGs for both architectures
   - Linux packages (AppImage, deb, rpm)
   - Android APK
3. Download and test artifacts on your target platforms
4. Update the release notes if needed

## Manual Release

If you need to manually trigger a release without creating a tag:

1. Go to the **Actions** tab
2. Select the "Release" workflow
3. Click "Run workflow"
4. Enter the version tag (e.g., `v1.0.0`)
5. Click "Run workflow"

## Configuration

### Required Secrets

For the release workflow to work properly, configure these GitHub secrets:

- `TAURI_SIGNING_PRIVATE_KEY` (optional): For code signing
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (optional): Password for the signing key

### Setting Up Secrets

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Add each required secret

### Code Signing

#### macOS

To sign macOS applications:

1. Generate an Apple Developer certificate
2. Export the certificate and key
3. Convert to base64: `base64 -i certificate.p12 | pbcopy`
4. Add to GitHub secrets as `TAURI_SIGNING_PRIVATE_KEY`
5. Add the password as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

#### Windows

Windows code signing is handled through the Tauri action. Configure:

1. `TAURI_SIGNING_PRIVATE_KEY`: Your Windows certificate
2. `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: Certificate password

## Troubleshooting

### Build Failures

**Rust compilation errors:**
- Check that all dependencies are compatible
- Verify Cargo.toml versions
- Review the error logs in GitHub Actions

**Frontend build errors:**
- Ensure TypeScript compiles locally: `yarn tsc`
- Check that all dependencies are in package.json
- Verify Node.js version compatibility

**Android build errors:**
- Verify NDK version (27.1.12297006)
- Check Android SDK setup
- Ensure all Android targets are configured

### Missing Artifacts

If artifacts are missing from the release:
- Check the GitHub Actions logs for errors
- Verify that the build step completed successfully
- Ensure file paths match the upload configuration

### Release Not Publishing

If the release stays as a draft:
- Check the `publish-release` job logs
- Verify all previous jobs completed successfully
- Manually publish from the GitHub releases page

## Version Rollback

If you need to roll back a release:

```bash
# Delete the tag locally
git tag -d v1.0.0

# Delete the tag remotely
git push origin :refs/tags/v1.0.0

# Delete the GitHub release manually from the web interface
```

## Best Practices

1. **Test locally first**: Always test builds on your development machine before releasing
2. **Update changelog**: Keep the CHANGELOG.md up to date with user-facing changes
3. **Semantic versioning**: Follow semver (MAJOR.MINOR.PATCH)
4. **Pre-releases**: Use tags like `v1.0.0-beta.1` for pre-releases
5. **Release notes**: Add detailed release notes after the automated release is published
6. **Test artifacts**: Download and test at least one artifact from each platform

## CI/CD Pipeline

The project includes a separate CI workflow that runs on every push and pull request:

- Lints TypeScript and Rust code
- Runs tests
- Builds the application (without creating releases)
- Runs on all platforms to catch platform-specific issues early

This ensures that only tested, working code gets released.

## Support

For issues with the release process:
- Check the [GitHub Actions documentation](https://docs.github.com/en/actions)
- Review the [Tauri Action documentation](https://github.com/tauri-apps/tauri-action)
- Open an issue in the repository
