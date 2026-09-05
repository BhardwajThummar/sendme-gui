# Contributing

Thanks for considering a contribution to SendMe GUI.

## Dev setup

See the [Quick Start](README.md#quick-start) section of the README for
prerequisites and running the app locally.

## Before opening a PR

CI runs these checks on every push — run them locally first:

```bash
# TypeScript
yarn tsc --noEmit

# Rust, from src-tauri/
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Full build
yarn build
```

## Making changes

1. Create a feature branch off `main`.
2. Make your changes, keeping them scoped to the issue/feature at hand.
3. Update [CHANGELOG.md](CHANGELOG.md) under `[Unreleased]`.
4. Open a pull request describing what changed and why.

## Reporting bugs / requesting features

Use the issue templates under **New Issue** on GitHub.

## Security issues

Do not open a public issue for security vulnerabilities — see
[SECURITY.md](SECURITY.md).
