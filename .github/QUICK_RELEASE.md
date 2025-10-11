# Quick Release Guide

## TL;DR - Create a Release

```bash
# 1. Bump version (choose one)
yarn version:patch  # 0.1.0 -> 0.1.1
yarn version:minor  # 0.1.0 -> 0.2.0
yarn version:major  # 0.1.0 -> 1.0.0

# 2. Push to GitHub
git push origin tauri-v2
git push origin v0.1.1  # Use your new version

# 3. Wait for GitHub Actions to build (15-30 minutes)
# 4. Check releases page for your build artifacts
```

That's it! The rest is automated.

## What Gets Built

When you push a tag, GitHub Actions automatically builds:

### Desktop Applications
- **Windows**: `.exe` installer and `.msi` package
- **macOS Intel**: `.dmg` and `.app.tar.gz`
- **macOS Apple Silicon**: `.dmg` and `.app.tar.gz`
- **Linux**: `.AppImage`, `.deb`, and `.rpm`

### Mobile Applications
- **Android**: Universal APK

## Monitoring the Build

1. Go to: `https://github.com/YOUR_USERNAME/YOUR_REPO/actions`
2. Click on the latest "Release" workflow
3. Watch the progress of each build job

## If Something Goes Wrong

### Build Failed?
1. Check the error in GitHub Actions logs
2. Fix the issue in your code
3. Delete the tag: `git tag -d v0.1.1 && git push origin :refs/tags/v0.1.1`
4. Delete the GitHub release from the web UI
5. Fix the issue, commit, and try again

### Missing Platform?
- Check the specific platform's job in GitHub Actions
- Review the error logs for that platform
- Common issues: missing dependencies, signing certificates, or platform-specific code errors

### Release is a Draft?
- This is intentional! The release starts as a draft
- Once all builds complete, it's automatically published
- If stuck as draft, check the `publish-release` job logs

## Pre-release Testing Checklist

Before creating a release tag, verify:

- [ ] All tests pass locally
- [ ] TypeScript compiles: `yarn tsc --noEmit`
- [ ] Rust code compiles: `cd src-tauri && cargo check`
- [ ] App runs in development: `yarn tauri dev`
- [ ] CHANGELOG.md is updated
- [ ] All changes are committed and pushed

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **Patch** (0.1.0 → 0.1.1): Bug fixes only
- **Minor** (0.1.0 → 0.2.0): New features, backwards compatible
- **Major** (0.1.0 → 1.0.0): Breaking changes

## Common Commands

```bash
# Check current version
yarn version:check

# Test build locally before releasing
yarn build

# Test Android build locally
yarn android:build

# View recent tags
git tag -l

# Delete a tag (if you made a mistake)
git tag -d v0.1.1
git push origin :refs/tags/v0.1.1
```

## Getting Help

- Full documentation: [RELEASE.md](RELEASE.md)
- Versioning guide: [../VERSIONING.md](../VERSIONING.md)
- Changelog: [../CHANGELOG.md](../CHANGELOG.md)
